use std::{
    mem::MaybeUninit,
    ops::{Deref, DerefMut},
};

use super::{bitset::BitSet, *};
pub mod children; 
pub mod iterators;
#[cfg(test)]
pub mod tests;

pub use iterators::*;
pub use children::*;

/// # Description
/// You can use this to build trees, for easy mutations
pub struct FlatForest<Tag, T> {
    children: Vec<Children<Tag>>,
    parent: Vec<Ptr<Tag>>,
    data: Vec<MaybeUninit<T>>,
    free_pool_header: Ptr<Tag>,
    needs_drop: BitSet,
}

impl<Tag: Copy, T> FlatForest<Tag, T> {
    pub fn new() -> Self {
        Self {
            parent: vec![],
            children: vec![],
            data: vec![],
            free_pool_header: Ptr::null(),
            needs_drop: BitSet::new().with_capacity(512),
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

    pub fn allocate_node(&mut self, data: T) -> Ptr<Tag> {
        if Ptr::null() != self.free_pool_header {
            let reclaimed_pointer = self.free_pool_header;
            let next_recycled_node = self
                .children
                .get(reclaimed_pointer.to_index())
                .and_then(|children| children.first())
                .copied()
                .unwrap_or(Ptr::null());

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

            self.free_pool_header = next_recycled_node;
            reclaimed_pointer
        } else {
            self.children.push(Children::new());
            self.data.push(MaybeUninit::new(data));
            self.parent.push(Ptr::null());
            self.needs_drop.push_bit(0xff);
            Ptr::from(self.children.len() - 1)
        }
    }

    pub fn free(&mut self, root: Ptr<Tag>) {
        if root == Ptr::null() {
            return;
        }
        let len = self.get_children(root).len();
        for k in 0..len {
            let child = self.get_children(root)[k];
            self.free(child);
        }
        unsafe {
            self.shallow_free(root);
        }
    }

    pub fn free_iter<'a>(&'a self) -> FreedNodesIter<'a, Tag, T> {
        FreedNodesIter::with_node(self.free_pool_header, self)
    }

    /// # Safety
    /// should call `free` instead of this because it only shallow-ly
    /// frees the node at `ptr`. Whatever contents were put inside `Ptr` will
    /// be dropped in place and the Node gets recycled/pooled
    pub unsafe fn shallow_free(&mut self, ptr: Ptr<Tag>) {
        if ptr != Ptr::null() {
            let old_free = self.free_pool_header;
            let cur_free = ptr;

            {
                let cur_children = &mut self.children[cur_free.to_index()];
                if cur_children.is_empty() {
                    cur_children.push(Ptr::null());
                }
            }

            *self
                .children
                .get_mut(cur_free.to_index())
                .unwrap()
                .first_mut()
                .unwrap() = old_free;

            if std::mem::needs_drop::<T>() {
                unsafe {
                    let data_ptr = self
                        .data
                        .get_mut(cur_free.to_index())
                        .expect("Failed to deref data")
                        .as_mut_ptr();
                    // clean up droppable data
                    std::ptr::drop_in_place(data_ptr);
                }
                // we've dropped the data in the node so flag the node as `dropped`
                self.needs_drop.set_bit(cur_free.to_index(), 0);
            }

            self.free_pool_header = cur_free;
        }
    }
}
impl<Tag, T> Drop for FlatForest<Tag, T> {
    fn drop(&mut self) {
        if !std::mem::needs_drop::<T>() {
            return;
        }
        let check_bit_in_block = |block: u64, bit_idx: u64| block & (1 << bit_idx);

        for (block_idx, block) in self.needs_drop.blocks_iter().copied().enumerate() {
            if block == 0 {
                continue;
            }

            let global_bit_idx_base = 64 * block_idx;
            for bit_idx in 0..64 {
                if check_bit_in_block(block, bit_idx as u64) > 0 {
                    unsafe {
                        let global_bit_idx = global_bit_idx_base + bit_idx;
                        std::ptr::drop_in_place(self.data[global_bit_idx].as_mut_ptr());
                    }
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

