use super::*; 



pub struct FreeCache<Tag> {
    buffer: RingBuffer<[BlockIdx; 16]>,
    _tag: PhantomData<Tag>,
}


impl<Tag> FreeCache<Tag> {
    pub fn new() -> Self {
        Self {
            buffer: RingBuffer::with_backing([BlockIdx(0); 16]),
            _tag: PhantomData,
        }
    }
}

impl<Tag> Default for FreeCache<Tag> {
    fn default() -> Self {
        Self::new()
    }
}

impl<Tag> Deref for FreeCache<Tag> {
    type Target = RingBuffer<[BlockIdx; 16]>;
    fn deref(&self) -> &Self::Target {
        &self.buffer
    }
}

impl<Tag> DerefMut for FreeCache<Tag> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.buffer
    }
}
