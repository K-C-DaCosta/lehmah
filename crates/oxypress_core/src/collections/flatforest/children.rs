use super::*;

pub struct Children<Tag> {
    inner: Vec<Ptr<Tag>>,
}

impl<Tag> Clone for Children<Tag> {
    fn clone(&self) -> Self {
        Self {
            inner: self.inner.clone(),
        }
    }
}

impl<Tag> Children<Tag> {
    pub fn new() -> Self {
        Self { inner: vec![] }
    }
}
impl<P> Default for Children<P> {
    fn default() -> Self {
        Self::new()
    }
}
impl<Tag> Deref for Children<Tag> {
    type Target = Vec<Ptr<Tag>>;
    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}
impl<Tag> DerefMut for Children<Tag> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.inner
    }
}
