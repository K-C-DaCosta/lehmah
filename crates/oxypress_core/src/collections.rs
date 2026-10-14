use std::marker::PhantomData;

pub mod bitset;
pub mod flatforest;
pub mod ringbuffer;

#[derive(Debug)]
pub struct Ptr<Tag> {
    data: usize,
    _tag: PhantomData<Tag>,
}
impl<Tag> Ptr<Tag> {
    pub fn null() -> Self {
        Self {
            data: !0,
            _tag: PhantomData,
        }
    }
    pub fn zero() -> Self {
        Self {
            data: 0,
            _tag: PhantomData,
        }
    }
    pub fn offset(mut self, offset: usize) -> Self {
        self.data += offset;
        self
    }
    pub fn to_index(&self) -> usize {
        self.data
    }
}
impl<Tag> PartialEq for Ptr<Tag> {
    fn eq(&self, other: &Self) -> bool {
        self.data == other.data
    }
}

impl<Tag> Eq for Ptr<Tag> {}

impl<Tag> PartialOrd for Ptr<Tag> {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl<Tag> Ord for Ptr<Tag> {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.data.cmp(&other.data)
    }
}
impl<Tag> Clone for Ptr<Tag> {
    fn clone(&self) -> Self {
        *self
    }
}
impl<Tag> Copy for Ptr<Tag> {}

impl<Tag> From<usize> for Ptr<Tag> {
    fn from(value: usize) -> Self {
        Self {
            data: value,
            _tag: PhantomData,
        }
    }
}
