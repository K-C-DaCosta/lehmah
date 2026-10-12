use super::*;

#[derive(Copy, Clone, Default, PartialEq, Eq)]
pub struct BitBlock {
    data: u64,
}
impl BitBlock {
    pub const fn bit_capacity() -> usize {
        u64::BITS as usize
    }

    pub fn calc_position(bit_position: usize) -> BitSetIndex {
        BitSetIndex {
            block_idx: bit_position / Self::bit_capacity(),
            bit_idx: bit_position % Self::bit_capacity(),
        }
    }

    pub fn is_empty(&self) -> bool {
        self.data == 0
    }
    pub fn get_bit(&self, idx: usize) -> u64 {
        (self.data >> idx) & 1
    }

    pub fn check_bit(&self, idx: usize) -> bool {
        self.get_bit(idx) > 0
    }

    pub fn clear_bit(&mut self, idx: usize) {
        self.data &= !(1 << idx);
    }
    pub fn set_bit(&mut self, idx: usize, val: u64) {
        let bit_val = (val > 0) as u64;
        self.clear_bit(idx);
        self.data |= bit_val << idx;
    }

    pub fn zero() -> Self {
        Self { data: 0 }
    }
    pub fn bits(&self) -> u64 {
        self.data
    }
}
