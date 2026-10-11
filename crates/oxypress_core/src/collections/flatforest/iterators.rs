use super::*;

pub struct FreedNodesIter<'a, Tag, T> {
    free_node: Ptr<Tag>,
    node_world: &'a FlatForest<Tag, T>,
}
impl<'a, Tag, T> FreedNodesIter<'a, Tag, T> {
    pub fn with_node(free_node: Ptr<Tag>, node_world: &'a FlatForest<Tag, T>) -> Self {
        Self {
            free_node,
            node_world,
        }
    }
}

impl<'a, Tag, T> Iterator for FreedNodesIter<'a, Tag, T>
where
    Tag: Copy,
{
    type Item = Ptr<Tag>;
    fn next(&mut self) -> Option<Self::Item> {
        let cur = self.free_node;
        let node_wold = self.node_world;
        (cur != Ptr::null()).then(|| {
            let next = node_wold.get_children(cur).first().copied().unwrap();
            self.free_node = next;
            cur
        })
    }
}
