#[derive(Copy,Clone,Debug)]
pub enum RingError {
    BufferFull,
    BufferEmpty,
}

pub type Result<T> = std::result::Result<T, RingError>;

pub struct RingBuffer<M> {
    memory: M,
    rear: usize,
    front: usize,
    len: usize,
}
impl<M, T> RingBuffer<M>
where
    M: std::ops::Index<usize, Output = T> + std::ops::IndexMut<usize> + HasLen,
{
    pub fn with_backing(memory: M) -> Self {
        Self {
            memory,
            front: 0,
            rear: 0,
            len: 0,
        }
    }

    pub fn len(&self) -> usize {
        self.len
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    pub fn is_full(&self) -> bool {
        self.len >= self.memory.len()
    }

    pub fn enqueue(&mut self, item: T) -> Result<()> {
        let mem_len = self.memory.len();
        if self.len >= self.memory.len() {
            return Err(RingError::BufferFull);
        }
        self.memory[self.rear % mem_len] = item;
        self.rear = (self.rear + 1) % mem_len;
        self.len += 1;
        Ok(())
    }

    pub fn dequeue(&mut self) -> Result<&T> {
        let mem_len = self.memory.len();
        if mem_len == 0 {
            return Err(RingError::BufferEmpty);
        }

        let fetched_item = &self.memory[self.front % mem_len];
        self.front = (self.front + 1) % mem_len;
        self.len -= 1;
        Ok(fetched_item)
    }

    pub fn peek_front(&self) -> &T {
        let mem_len = self.memory.len();
        &self.memory[self.front % mem_len]
    }

    pub fn release(self) -> M {
        self.memory
    }
}

pub trait HasLen {
    fn len(&self) -> usize;
    fn is_empty(&self) -> bool {
        self.len() == 0
    }
}

impl<T, const N: usize> HasLen for [T; N] {
    fn len(&self) -> usize {
        N
    }
}

impl<T> HasLen for [T] {
    fn len(&self) -> usize {
        self.len()
    }
}

impl<T> HasLen for Vec<T> {
    fn len(&self) -> usize {
        self.len()
    }
}
