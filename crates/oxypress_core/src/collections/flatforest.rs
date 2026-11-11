use std::{
    mem::MaybeUninit,
    ops::{Deref, DerefMut},
    ptr::null,
};

use crate::collections::{
    bitset::{BitBlock, PopulatedBlockItem},
    ringbuffer::RingError,
};

use super::ringbuffer::RingBuffer;
use super::{bitset::BitSet, *};

pub mod children;
pub mod freecache;
pub mod iterators;
#[cfg(test)]
pub mod tests;

pub use children::*;
use freecache::*;
pub use iterators::*;

pub enum TakeError {
    AlreadyDropped,
}

#[derive(Copy, Clone, PartialEq)]
pub struct BlockIdx(pub usize);

/// # Description
/// You can use this to build trees, for easy mutations
pub struct FlatForest<Tag, T> {
    children: Vec<Children<Tag>>,
    parent: Vec<Ptr<Tag>>,
    data: Vec<MaybeUninit<T>>,
    is_freed: BitSet,
    needs_drop: BitSet,
    freed_cache: FreeCache<Tag>,
}

impl<Tag, T> FlatForest<Tag, T> {
    pub fn new() -> Self {
        Self {
            parent: vec![],
            children: vec![],
            data: vec![],
            is_freed: BitSet::new().with_capacity(512),
            needs_drop: BitSet::new().with_capacity(512),
            freed_cache: FreeCache::new(),
        }
    }

    pub fn get_children(&self, ptr: Ptr<Tag>) -> &Children<Tag> {
        &self.children[ptr.to_index()]
    }

    pub fn get_children_mut(&mut self, ptr: Ptr<Tag>) -> &mut Children<Tag> {
        &mut self.children[ptr.to_index()]
    }

    pub fn get_parent(&self, ptr: Ptr<Tag>) -> &Ptr<Tag> {
        &self.parent[ptr.to_index()]
    }

    pub fn get_parent_mut(&mut self, ptr: Ptr<Tag>) -> &mut Ptr<Tag> {
        &mut self.parent[ptr.to_index()]
    }

    pub fn get_data(&self, ptr: Ptr<Tag>) -> &T {
        unsafe {
            self.data[ptr.to_index()]
                .as_ptr()
                .as_ref()
                .expect("Failed to cast")
        }
    }

    pub fn get_data_mut(&mut self, ptr: Ptr<Tag>) -> &mut T {
        unsafe {
            self.data[ptr.to_index()]
                .as_mut_ptr()
                .as_mut()
                .expect("Failed to cast")
        }
    }

    fn update_freed_cache(&mut self) {
        let cache = &mut self.freed_cache;
        let is_free = &mut self.is_freed;

        // cache is still poll-able so skip update
        if !cache.is_empty() {
            return;
        }

        for PopulatedBlockItem { block_idx, .. } in is_free.populated_blocks() {
            if let Err(RingError::BufferFull) = cache.enqueue(BlockIdx(block_idx)) {
                break;
            }
        }
    }

    fn poll_cache(&mut self) -> Option<Ptr<Tag>> {
        let is_freed = &mut self.is_freed;
        let cache = &mut self.freed_cache;
        while !cache.is_empty() {
            let &BlockIdx(block_idx) = cache.front();
            let block = &mut is_freed[block_idx];
            if block.bits() > 0 {
                let bit_idx = block.get_lowest_bit_idx();
                let freed_node = Ptr::from(block_idx * BitBlock::bit_capacity() + bit_idx);
                block.clear_lowest_bit();
                return Some(freed_node);
            }
            cache.dequeue().expect("Failed to dequeue");
        }
        None
    }
    fn walk_iter<'a, F, Item>(
        &'a self,
        root: Ptr<Tag>,
        get_item: F,
    ) -> impl Iterator<Item = (Ptr<Tag>, Item)>
    where
        F: Fn(&'a Self, Ptr<Tag>) -> Item,
    {
        #[derive(Debug)]
        struct StackFrame<Tag> {
            root: Ptr<Tag>,
            child_idx: usize,
            len: usize,
            visited: bool,
        }
        impl<Tag> Clone for StackFrame<Tag> {
            fn clone(&self) -> Self {
                *self
            }
        }
        impl<Tag> Copy for StackFrame<Tag> {}
        let mut stack: Vec<StackFrame<Tag>> = vec![StackFrame {
            root,
            child_idx: 0,
            len: self.get_children(root).len(),
            visited: false,
        }];

        std::iter::from_fn(move || {
            while let Some(mut frame) = stack.pop() {
                let child_idx = frame.child_idx;
                let visited = frame.visited;
                if frame.child_idx < frame.len {
                    frame.child_idx += 1;
                    frame.visited = true;
                    stack.push(frame);

                    let children = self.get_children(frame.root);
                    let next_root = children[child_idx];
                    let next_len = self.get_children(next_root).len();

                    stack.push(StackFrame {
                        root: next_root,
                        child_idx: 0,
                        len: next_len,
                        visited: false,
                    });
                }
                if !visited {
                    let item = get_item(self, frame.root);
                    return Some((frame.root, item));
                }
            }
            None
        })
    }
    pub fn iter(&self, root: Ptr<Tag>) -> impl Iterator<Item = (Ptr<Tag>, &T)> {
        self.walk_iter(root, |forest, ptr| forest.get_data(ptr))
    }

    pub fn iter_mut(&mut self, root: Ptr<Tag>) -> impl Iterator<Item = (Ptr<Tag>, &mut T)> {
        let self_ptr = self as *mut Self;
        self.walk_iter(root, move |_, ptr| unsafe { (*self_ptr).get_data_mut(ptr) })
    }
    pub fn iter_inclusive(
        &self,
        root: Ptr<Tag>,
        lbound: Ptr<Tag>,
        ubound: Ptr<Tag>,
    ) -> impl Iterator<Item = (Ptr<Tag>, &T)> {
        enum IterState {
            Start,
            Collect,
            Terminate,
        }

        let mut iter_state = IterState::Start;
        let mut node_iter = self.iter(root);
        std::iter::from_fn(move || {
            #[allow(clippy::while_let_on_iterator)]
            while let Some((ptr, data)) = node_iter.next() {
                match iter_state {
                    IterState::Start => {
                        if ptr == lbound {
                            iter_state = IterState::Collect;
                            return Some((ptr, data));
                        }
                    }
                    IterState::Collect => {
                        if ptr == ubound {
                            iter_state = IterState::Terminate;
                        }
                        return Some((ptr, data));
                    }
                    IterState::Terminate => {
                        return None;
                    }
                }
            }
            None
        })
    }

    pub fn iter_inclusive_mut(
        &mut self,
        root: Ptr<Tag>,
        lbound: Ptr<Tag>,
        ubound: Ptr<Tag>,
    ) -> impl Iterator<Item = (Ptr<Tag>, &mut T)> {
        let self_ptr = self as *mut Self;
        self.iter_inclusive(root, lbound, ubound)
            .map(move |(ptr, _)| unsafe {
                let self_mut_ref = &mut *self_ptr;
                let data = self_mut_ref.get_data_mut(ptr);
                (ptr, data)
            })
    }

    pub fn iter_exclusive(
        &self,
        root: Ptr<Tag>,
        lbound: Ptr<Tag>,
        ubound: Ptr<Tag>,
    ) -> impl Iterator<Item = (Ptr<Tag>, &T)> {
        enum IterState {
            Start,
            Collect,
        }

        let mut iter_state = IterState::Start;
        let mut node_iter = self.iter(root);
        std::iter::from_fn(move || {
            #[allow(clippy::while_let_on_iterator)]
            while let Some((ptr, data)) = node_iter.next() {
                match iter_state {
                    IterState::Start => {
                        if ptr == lbound {
                            iter_state = IterState::Collect;
                        }
                    }
                    IterState::Collect => {
                        return if ptr == ubound {
                            None
                        } else {
                            Some((ptr, data))
                        };
                    }
                }
            }
            None
        })
    }

    pub fn iter_exclusive_mut(
        &mut self,
        root: Ptr<Tag>,
        lbound: Ptr<Tag>,
        ubound: Ptr<Tag>,
    ) -> impl Iterator<Item = (Ptr<Tag>, &mut T)> {
        let self_ptr = self as *mut Self;
        self.iter_exclusive(root, lbound, ubound)
            .map(move |(ptr, _)| unsafe {
                let self_mut_ref = &mut *self_ptr;
                let data = self_mut_ref.get_data_mut(ptr);
                (ptr, data)
            })
    }

    pub fn allocate_node(&mut self, data: T) -> Ptr<Tag> {
        self.update_freed_cache();
        if let Some(reclaimed_pointer) = self.poll_cache() {
            // initalize data with new value
            self.data
                .get_mut(reclaimed_pointer.to_index())
                .expect("Failed to derefence pointer")
                .write(data);

            // reset children to zero
            self.children
                .get_mut(reclaimed_pointer.to_index())
                .unwrap()
                .clear();
            // re-initalize parent pointer
            self.parent[reclaimed_pointer.to_index()] = Ptr::null();
            // flag the node as drop-able
            self.needs_drop.set_bit(reclaimed_pointer.to_index(), 1);
            self.is_freed.set_bit(reclaimed_pointer.to_index(), 0);

            reclaimed_pointer
        } else {
            self.children.push(Children::new());
            self.data.push(MaybeUninit::new(data));
            self.parent.push(Ptr::null());
            self.needs_drop.push_bit(0xff);
            self.is_freed.push_bit(0x00);
            Ptr::from(self.children.len() - 1)
        }
    }

    pub fn add_child(&mut self, root: Ptr<Tag>, child: Ptr<Tag>) {
        self.children[root.to_index()].push(child);
        self.parent[child.to_index()] = root;
    }
    pub fn add_child_at(
        &mut self,
        root: Ptr<Tag>,
        child: Ptr<Tag>,
        target: Ptr<Tag>,
        offset: isize,
    ) {
        let children = &mut self.children[root.to_index()];

        let target_position = children
            .iter()
            .enumerate()
            .find(|&(_idx, &node)| node == target)
            .map(|(idx, _node)| idx)
            .expect("Failed to find target") as isize;

        children.insert((target_position + offset) as usize, child);
        self.parent[child.to_index()] = root;
    }

    pub fn free(&mut self, root: Ptr<Tag>) {
        self.free_and_then(root, |_| ());
    }

    pub fn free_and_then<F>(&mut self, root: Ptr<Tag>, mut after_free: F)
    where
        F: FnMut(Ptr<Tag>),
    {
        let &parent = self.get_parent(root);
        if !parent.is_null() {
            let parent_child_list = self.get_children_mut(parent);
            let root_location = parent_child_list
                .iter()
                .copied()
                .enumerate()
                .find(|&(_, e)| e == root)
                .map(|(idx, _)| idx)
                .expect("Failed to find root in parent");
            parent_child_list.remove(root_location);
        }
        self.free_helper(root, &mut after_free);
    }

    fn free_helper<F>(&mut self, root: Ptr<Tag>, after_free: &mut F)
    where
        F: FnMut(Ptr<Tag>),
    {
        if root == Ptr::null() {
            return;
        }
        let len = self.get_children(root).len();
        unsafe {
            self.shallow_free(root);
        }
        after_free(root);
        for k in 0..len {
            let child = self.get_children(root)[k];
            self.free_helper(child, after_free);
        }
    }

    pub fn traverse_all_mut<F>(&mut self, root: Ptr<Tag>, mut on_root: F)
    where
        F: FnMut(&mut Self, Ptr<Tag>) -> bool,
    {
        let _ = self.traverse_all_helper_mut(root, &mut on_root);
    }

    fn traverse_all_helper_mut<F>(&mut self, root: Ptr<Tag>, on_root: &mut F) -> bool
    where
        F: FnMut(&mut Self, Ptr<Tag>) -> bool,
    {
        if root == Ptr::null() {
            return false;
        }
        if (on_root)(self, root) {
            return true;
        }
        let child_count = self.get_children(root).len();
        for child_idx in 0..child_count {
            let child = self.get_children(root)[child_idx];
            if self.traverse_all_helper_mut(child, on_root) {
                return true;
            }
        }
        false
    }

    /// # Safety
    /// Frees a single node and drops content of the node. Not safe to call
    /// without knowing exactly what you're doing because if done wrong it can
    /// break tree traversals
    pub unsafe fn shallow_free(&mut self, ptr: Ptr<Tag>) {
        self.is_freed.set_bit(ptr.to_index(), 0xff);
        if std::mem::needs_drop::<T>() && self.needs_drop.check_bit(ptr.to_index()) {
            unsafe {
                self.data
                    .get_mut(ptr.to_index())
                    .expect("Failed to derefence")
                    .assume_init_drop();
            }
            self.needs_drop.set_bit(ptr.to_index(), 0x00);
        }
    }

    pub fn free_iter(&self) -> impl Iterator<Item = Ptr<Tag>> {
        self.is_freed.populated_blocks().flat_map(|elem| {
            let block_idx = elem.block_idx;
            let mut block = elem.block.bits();
            std::iter::from_fn(move || {
                if block == 0 {
                    return None;
                }
                let bit_idx = block.trailing_zeros();
                block &= block - 1;
                Some(Ptr::from(
                    block_idx * BitBlock::bit_capacity() + bit_idx as usize,
                ))
            })
        })
    }

    pub fn drain<F>(&mut self, mut before_drop: F)
    where
        F: FnMut(Ptr<Tag>),
    {
        if std::mem::needs_drop::<T>() {
            for (block_idx, block) in self.needs_drop.blocks_iter_mut().enumerate() {
                if block.is_empty() {
                    continue;
                }
                let global_bit_idx_base = BitBlock::bit_capacity() * block_idx;
                while block.bits() > 0 {
                    let bit_idx = block.get_lowest_bit_idx();
                    block.clear_lowest_bit();
                    unsafe {
                        let global_bit_idx = global_bit_idx_base + bit_idx;
                        before_drop(Ptr::from(global_bit_idx));
                        std::ptr::drop_in_place(self.data[global_bit_idx].as_mut_ptr());
                    }
                }
            }
        }

        self.needs_drop.blocks_iter_mut().for_each(|block| {
            block.inner = 0;
        });

        // all nodes are recycleable now
        self.is_freed
            .blocks_iter_mut()
            .for_each(|block| block.inner = !0);
        // makes sure cache isn't stale
        self.freed_cache.clear();
    }
}

impl<Tag, T: PartialEq> PartialEq for FlatForest<Tag, T> {
    fn eq(&self, other: &Self) -> bool {
        self.data.len() == other.data.len()
            && self.parent.len() == other.parent.len()
            && self.children.len() == other.children.len()
            && self
                .data
                .iter()
                .zip(other.data.iter())
                .enumerate()
                .all(|(idx, (left, right))| {
                    self.is_freed.check_bit(idx) == other.is_freed.check_bit(idx)
                        && self.needs_drop.check_bit(idx) == other.needs_drop.check_bit(idx)
                        && self.needs_drop.check_bit(idx)
                        && other.needs_drop.check_bit(idx)
                        && unsafe { left.assume_init_ref().eq(right.assume_init_ref()) }
                })
    }
}

impl<Tag, T> Drop for FlatForest<Tag, T> {
    fn drop(&mut self) {
        self.drain(|_| ());
    }
}

impl<Tag: Copy, T> Default for FlatForest<Tag, T> {
    fn default() -> Self {
        Self::new()
    }
}
