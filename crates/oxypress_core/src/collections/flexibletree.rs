use std::{
    mem::MaybeUninit,
    ops::{Deref, DerefMut},
};

use super::*;

// For easy mutations
pub struct FlexibleTree<Tag, T> {
    root: Ptr<Tag>,
    nodes: Vec<Node<Tag>>,
    data: Vec<MaybeUninit<T>>,
    free_pool: Ptr<Tag>,
}

impl<Tag:Copy, T> FlexibleTree<Tag, T> {
    pub fn new() -> Self {
        Self {
            root: Ptr::null(),
            nodes: vec![],
            data: vec![],
            free_pool: Ptr::null(),
        }
    }
    pub fn allocate_node(&mut self, data: T) -> Ptr<Tag> {
        if Ptr::null() != self.free_pool {
            let reclaimed_pointer = self.free_pool;
            let next_recycled_node = self
                .nodes
                .get(reclaimed_pointer.to_index())
                .and_then(|node| node.children.first())
                .copied()
                .unwrap_or(Ptr::null());

            // initalize data with new value
            self.data
                .get_mut(reclaimed_pointer.to_index())
                .expect("Failed to derefence pointer")
                .write(data);

            // reset children to zero
            self.nodes
                .get_mut(reclaimed_pointer.to_index())
                .unwrap()
                .children
                .clear();

            self.free_pool = next_recycled_node;
            reclaimed_pointer
        } else {
            self.nodes.push(Node::new());
            self.data.push(MaybeUninit::new(data));
            Ptr::from(self.nodes.len() - 1)
        }
    }

    fn free_individual_node(&mut self, ptr:Ptr<Tag> ) {
        if ptr != Ptr::null() {
            let old_free = self.free_pool;
            let cur_free = ptr;

            {
                let cur_children = &mut self.nodes[cur_free.to_index()].children;
                if cur_children.is_empty() {
                    cur_children.push(Ptr::null());
                }
            }

            *self
                .nodes
                .get_mut(cur_free.to_index())
                .unwrap()
                .children
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
                    // clear slot just as an extra step
                    data_ptr.write_bytes(0, size_of::<T>());
                }
            }

            self.free_pool = cur_free;
        }
    }
}

pub struct Node<Tag> {
    children: Vec<Ptr<Tag>>,
}
impl<Tag> Node<Tag> {
    pub fn new() -> Self {
        Self { children: vec![] }
    }
}
impl<P> Default for Node<P> {
    fn default() -> Self {
        Self::new()
    }
}


