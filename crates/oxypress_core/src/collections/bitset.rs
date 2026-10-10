pub struct BitSet {
    bit_cursor: usize,
    blocks: Vec<u64>,
}
impl BitSet {
    pub fn new() -> Self {
        Self {
            bit_cursor: 0,
            blocks: vec![],
        }
    }
    pub fn with_capacity(mut self, bits: usize) -> Self {
        self.blocks.resize((bits / 64) + (bits % 64).clamp(0, 1), 0);
        self
    }
    pub fn bit_capacity(&self) -> usize {
        self.blocks.len() * 64
    }

    pub fn len_blocks(&self) -> usize {
        (self.bit_cursor + 63) / 64
    }
    /// # Description
    /// returns the number of bits the set is currently holding
    pub fn len(&self) -> usize {
        self.bit_cursor
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    pub fn push_bit(&mut self, bit_value: u8) {
        if self.bit_cursor <= self.blocks.len() * 64 {
            self.blocks.push(0);
        }
        let bit_value = (bit_value > 0) as u64;
        let block_idx = self.bit_cursor / 64;
        let bit_position = self.bit_cursor % 64;
        self.blocks[block_idx] |= bit_value << bit_position;

        self.bit_cursor+=1;
    }
    pub fn get_bit(&self, idx: usize) -> u64 {
        let block_idx = idx / 64;
        let bit_idx = idx % 64;
        (self.blocks[block_idx] >> bit_idx) & 1
    }
    pub fn set_bit(&mut self, idx: usize, bit_value: u64) {
        let bit_value = (bit_value > 0) as u64;
        let block_idx = idx / 64;
        let bit_idx = idx % 64;
        let block = self.blocks[block_idx];
        let cleared_block = block & (!(1 << bit_idx));
        self.blocks[block_idx] = cleared_block | (bit_value << bit_idx);
    }

    pub fn blocks_iter(&self) -> impl Iterator<Item = &u64> {
        self.blocks.iter().take(self.len_blocks())
    }
}
impl Default for BitSet {
    fn default() -> Self {
        Self::new()
    }
}
