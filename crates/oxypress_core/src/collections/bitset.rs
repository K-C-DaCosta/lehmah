pub mod bitblock;
pub mod iterators;
#[cfg(test)]
pub mod tests;
pub use bitblock::*;
pub use iterators::*;

pub struct BitSetIndex {
    block_idx: usize,
    bit_idx: usize,
}
pub struct BitSet {
    bit_cursor: usize,
    blocks: Vec<BitBlock>,
}

impl BitSet {
    pub fn new() -> Self {
        Self {
            bit_cursor: 0,
            blocks: vec![],
        }
    }
    pub fn with_capacity(mut self, bits: usize) -> Self {
        self.blocks
            .resize(bits.div_ceil(BitBlock::bit_capacity()), BitBlock::zero());
        self
    }
    pub fn bit_capacity(&self) -> usize {
        self.blocks.len() * BitBlock::bit_capacity()
    }

    pub fn len_blocks(&self) -> usize {
        self.bit_cursor.div_ceil(BitBlock::bit_capacity())
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
        if self.bit_cursor <= self.blocks.len() * BitBlock::bit_capacity() {
            self.blocks.push(BitBlock::zero());
        }
        let BitSetIndex { block_idx, bit_idx } = BitBlock::calc_position(self.bit_cursor);
        self.blocks[block_idx].set_bit(bit_idx, bit_value as u64);
        self.bit_cursor += 1;
    }
    pub fn check_bit(&self, idx: usize) -> bool {
        self.get_bit(idx) > 0
    }

    pub fn get_bit(&self, idx: usize) -> u64 {
        let BitSetIndex { block_idx, bit_idx } = BitBlock::calc_position(idx);
        self.blocks[block_idx].get_bit(bit_idx)
    }

    pub fn set_bit(&mut self, idx: usize, bit_value: u64) {
        let BitSetIndex { block_idx, bit_idx } = BitBlock::calc_position(idx);
        self.blocks[block_idx].set_bit(bit_idx, bit_value);
    }

    pub fn blocks_iter(&self) -> impl Iterator<Item = &BitBlock> {
        self.blocks.iter().take(self.len_blocks())
    }

    pub fn clear(&mut self) {
        self.blocks.clear();
        self.bit_cursor = 0;
    }

    pub fn populated_blocks<'a>(&'a self) -> impl Iterator<Item = PopulatedBlockItem<'a>> {
        PopulatedBlocksIter::from_bitset(self)
    }
}
impl Default for BitSet {
    fn default() -> Self {
        Self::new()
    }
}
impl<T: Into<u8>> FromIterator<T> for BitSet {
    fn from_iter<I: IntoIterator<Item = T>>(iter: I) -> Self {
        iter.into_iter().fold(Self::new(), |mut acc, elem| {
            acc.push_bit(elem.into());
            acc
        })
    }
}
impl std::ops::Index<usize> for BitSet {
    type Output = BitBlock;
    fn index(&self, index: usize) -> &Self::Output {
        &self.blocks[index]
    }
}
impl std::ops::IndexMut<usize> for BitSet {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        &mut self.blocks[index]
    }
}

