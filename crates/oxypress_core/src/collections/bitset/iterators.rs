use super::*; 

pub struct PopulatedBlocksIter<'a> {
    bitset: &'a BitSet,
    block_cursor: usize,
}
impl<'a> PopulatedBlocksIter<'a> {
    pub fn from_bitset(bitset: &'a BitSet) -> Self {
        Self {
            block_cursor: 0,
            bitset
        }
    }
}

pub struct PopulatedBlockItem<'a> {
    pub block: &'a BitBlock,
    pub block_idx: usize,
}

impl<'a> Iterator for PopulatedBlocksIter<'a> {
    type Item = PopulatedBlockItem<'a>;
    fn next(&mut self) -> Option<Self::Item> {
        let bitset = &self.bitset;
        let block_count = bitset.len_blocks();
        let mut block_cursor = self.block_cursor;

        while block_cursor < block_count && bitset[block_cursor].bits() == 0 {
            block_cursor += 1;
        }

        let result =
            ((block_cursor < block_count) && (bitset[block_cursor].bits() > 0)).then(|| {
                PopulatedBlockItem {
                    block: &bitset[block_cursor],
                    block_idx: block_count,
                }
            });

        self.block_cursor = block_cursor + 1;

        result
    }
}


