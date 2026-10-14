use std::{
    mem::MaybeUninit,
    ops::{Deref, DerefMut},
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

impl<Tag: Copy, T> FlatForest<Tag, T> {
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

    pub fn get_parent_mut(&mut self, ptr: Ptr<Tag>) -> &mut Children<Tag> {
        &mut self.children[ptr.to_index()]
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

    pub fn free(&mut self, root: Ptr<Tag>) {
        if root == Ptr::null() {
            return;
        }
        let len = self.get_children(root).len();
        unsafe {
            self.shallow_free(root);
        }
        for k in 0..len {
            let child = self.get_children(root)[k];
            self.free(child);
        }
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

}
impl<Tag, T> Drop for FlatForest<Tag, T> {
    fn drop(&mut self) {
        if !std::mem::needs_drop::<T>() {
            return;
        }
        for (block_idx, mut block) in self.needs_drop.blocks_iter().copied().enumerate() {
            if block.is_empty() {
                continue;
            }
            let global_bit_idx_base = BitBlock::bit_capacity() * block_idx;
            while block.bits() > 0 {
                let bit_idx = block.get_lowest_bit_idx();
                block.clear_lowest_bit();
                unsafe {
                    let global_bit_idx = global_bit_idx_base + bit_idx;
                    std::ptr::drop_in_place(self.data[global_bit_idx].as_mut_ptr());
                }
            }
        }
    }
}

impl<Tag: Copy, T> Default for FlatForest<Tag, T> {
    fn default() -> Self {
        Self::new()
    }
}
