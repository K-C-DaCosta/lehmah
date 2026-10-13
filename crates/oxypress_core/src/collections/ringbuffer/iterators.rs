use super::*;

pub struct RingBufferIterator<'a, M> {
    rb: &'a RingBuffer<M>,
    cursor: usize,
    times_moved: usize,
}
impl<'a, M> RingBufferIterator<'a, M> {
    pub fn with_ctx(rb: &'a RingBuffer<M>, cursor: usize) -> Self {
        Self {
            rb,
            cursor,
            times_moved: 0,
        }
    }
}

impl<'a, M, T> Iterator for RingBufferIterator<'a, M>
where
    M: std::ops::Index<usize, Output = T> + HasLen,
    T: 'a,
{
    type Item = &'a T;
    fn next(&mut self) -> Option<Self::Item> {
        let mem_len = self.rb.memory.len();
        let res = (self.times_moved < self.rb.len()).then(|| &self.rb.memory[self.cursor]);
        self.times_moved += 1;
        self.cursor = (self.cursor + 1) % mem_len;
        res
    }
}
