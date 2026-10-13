use super::*;

type IntegerKind = u64;

#[derive(Copy, Clone, Default, PartialEq, Eq)]
pub struct BitBlock {
    pub inner: IntegerKind,
}
impl BitBlock {
    pub const fn bit_capacity() -> usize {
        IntegerKind::BITS as usize
    }

    pub fn calc_position(bit_position: usize) -> BitSetIndex {
        BitSetIndex {
            block_idx: bit_position / Self::bit_capacity(),
            bit_idx: bit_position % Self::bit_capacity(),
        }
    }

    pub fn is_empty(&self) -> bool {
        self.inner == 0
    }

    pub fn get_bit(&self, idx: usize) -> u64 {
        (self.inner >> idx) & 1
    }

    pub fn check_bit(&self, idx: usize) -> bool {
        self.get_bit(idx) > 0
    }

    pub fn clear_bit(&mut self, idx: usize) {
        self.inner &= !(1 << idx);
    }

    pub fn set_bit(&mut self, idx: usize, val: IntegerKind) {
        let bit_val = (val > 0) as u64;
        self.clear_bit(idx);
        self.inner |= bit_val << idx;
    }

    pub fn zero() -> Self {
        Self { inner: 0 }
    }
    pub fn bits(&self) -> IntegerKind {
        self.inner
    }
}

impl From<IntegerKind> for BitBlock{
    fn from(value: IntegerKind) -> Self {
        Self{
            inner:value
        }
    }
}
