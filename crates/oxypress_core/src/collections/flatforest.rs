use std::{
    mem::MaybeUninit,
    ops::{Deref, DerefMut},
};

use super::ringbuffer::RingBuffer;
use super::{bitset::BitSet, *};
pub mod children;
pub mod iterators;
#[cfg(test)]
pub mod tests;

pub use children::*;
pub use iterators::*;

pub enum TakeError {
    AlreadyDropped,
}

pub struct FreeCache<Tag> {
    buffer: RingBuffer<[usize; 16]>,
    _tag: PhantomData<Tag>,
}

impl<Tag> FreeCache<Tag> {
    pub fn new() -> Self {
        Self {
            buffer: RingBuffer::with_backing([0; 16]),
            _tag: PhantomData,
        }
    }
}

impl  <Tag> Default for FreeCache<Tag> {
    fn default() -> Self {
        Self::new()
    }
}

impl<Tag> Iterator for FreeCache<Tag> {
    type Item = Ptr<Tag>;
    fn next(&mut self) -> Option<Self::Item> {
        unimplemented!("IDK fuck man")
    }
}

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
        unimplemented!("needs implementing")
    }

    pub fn allocate_node(&mut self, data: T) -> Ptr<Tag> {
        self.update_freed_cache();
        if let Some(reclaimed_pointer) = self.freed_cache.next() {
            let reclaimed_pointer = Ptr::null();

            if std::mem::needs_drop::<T>()
                && self.needs_drop.check_bit(reclaimed_pointer.to_index())
            {
                unsafe {
                    std::ptr::drop_in_place(
                        self.data
                            .get_mut(reclaimed_pointer.to_index())
                            .expect("Failed to derefence pointer")
                            .as_mut_ptr(),
                    );
                }
            }

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
        self.shallow_free(root);
        for k in 0..len {
            let child = self.get_children(root)[k];
            self.free(child);
        }
    }
    pub fn shallow_free(&mut self, ptr: Ptr<Tag>) {
        self.is_freed.set_bit(ptr.to_index(), 0xff);
    }
}
impl<Tag, T> Drop for FlatForest<Tag, T> {
    fn drop(&mut self) {
        if !std::mem::needs_drop::<T>() {
            return;
        }
        for (block_idx, block) in self.needs_drop.blocks_iter().copied().enumerate() {
            if block.is_empty() {
                continue;
            }

            let global_bit_idx_base = 64 * block_idx;
            for bit_idx in 0..64 {
                if block.check_bit(bit_idx) {
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
