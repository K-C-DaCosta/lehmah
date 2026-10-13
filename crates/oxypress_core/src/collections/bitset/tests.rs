use rand::RngExt;

use crate::{log::*, loggy};

use super::*;
#[test]
fn basic_test() {
    let mut result = [0, 0, 1, 1, 0, 0, 0, 1, 1, 1, 0, 1]
        .iter()
        .rev()
        .copied()
        .collect::<BitSet>();
    assert_eq!(result.blocks[0].bits(), 0b0000_0011_0001_1101);
    result.set_bit(0, 0);
    assert_eq!(result.blocks[0].bits(), 0b0000_0011_0001_1100);
    result.set_bit(8, 0);
    assert_eq!(result.blocks[0].bits(), 0b0000_0010_0001_1100);
    assert_eq!(result.len(), 12);

    result.clear();
    assert_eq!(
        result.blocks.len(),
        0,
        "Unexpected block len, expected empty"
    );
    assert_eq!(
        result.bit_cursor, 0,
        "Unexpected bit cursor, should be at start(0)"
    );
}

#[test]
fn populated_blocks_fuzz_test() {
    const MAX_ITERATIONS: usize = 20;
    const MAX_BLOCKS: u64 = 1024;
    const MIN_BLOCKS: u64 = 512;

    use rand::RngExt;
    let mut test_rand =
        <rand::rngs::SmallRng as rand::SeedableRng>::seed_from_u64(0x12abcd00aabbccdd);

    println!();
    for iter_count in 1..=MAX_ITERATIONS {
        let number_of_blocks = (test_rand.random::<u64>() % (MAX_BLOCKS - MIN_BLOCKS)) + MIN_BLOCKS;
        let mut expected_number_of_populated_chunks = 0;
        let mut expected_block_positions = vec![];

        let randomized_block_vec = {
            let mut v = vec![];
            while v.len() < number_of_blocks as usize {
                v.push(if (test_rand.random::<u64>() % 100) < 70 {
                    BitBlock::from(0)
                } else {
                    expected_number_of_populated_chunks += 1;
                    expected_block_positions.push(v.len());
                    BitBlock::from(!0)
                });
            }
            v
        };

        let bitset = randomized_block_vec
            .iter()
            .copied()
            .fold(BitSet::new(), |mut acc, val| {
                acc.blocks.push(val);
                acc.bit_cursor += BitBlock::bit_capacity();
                acc
            });

        let empty_blocks_ratio = bitset.blocks_iter().filter(|b| b.bits() == 0).count() as f32
            / bitset.len_blocks() as f32;

        let populated_block_positions = bitset
            .populated_blocks()
            .map(|a| a.block_idx)
            .collect::<Vec<_>>();

        assert_eq!(
            populated_block_positions.len(),
            expected_number_of_populated_chunks as usize,
            "Unexpected length of blocks"
        );
        assert_eq!(
            populated_block_positions.len(),
            expected_block_positions.len(),
            "Unexpected length discrepancy"
        );
        assert_eq!(
            populated_block_positions, expected_block_positions,
            "Unexpected list contents discrepancy"
        );

        loggy!(
            ConsoleColors::GREEN,
            "ITERATION PASSED iter_count = [{iter_count}] blocks generated = [{number_of_blocks}] empty_blocks_ratio = {} ",
            empty_blocks_ratio*100.0
        );
    }
}
