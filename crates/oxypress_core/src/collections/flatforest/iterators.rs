use crate::collections::bitset::BitBlock;

use super::*;

pub struct FreedBlocksIter<'a> {
    is_freed: &'a BitSet,
    block_cursor: usize,
}
impl<'a> FreedBlocksIter<'a> {
    pub fn from_forest(is_freed: &'a BitSet) -> Self {
        Self {
            block_cursor: 0,
            is_freed,
        }
    }
}

pub struct FreeBlockElem<'a> {
    pub block: &'a BitBlock,
    pub block_idx: usize,
}

impl<'a > Iterator for FreedBlocksIter<'a >
{
    type Item = FreeBlockElem<'a>;
    fn next(&mut self) -> Option<Self::Item> {
        let is_freed = &self.is_freed;
        let block_count = is_freed.len_blocks();
        let mut block_cursor = self.block_cursor;

        while block_cursor < block_count && is_freed[block_cursor].bits() == 0 {
            block_cursor += 1;
        }

        let result =
            ((block_cursor < block_count) && (is_freed[block_cursor].bits() > 0)).then(|| {
                FreeBlockElem {
                    block: &is_freed[block_cursor],
                    block_idx: block_count,
                }
            });

        self.block_cursor = block_cursor + 1;

        result
    }
}
