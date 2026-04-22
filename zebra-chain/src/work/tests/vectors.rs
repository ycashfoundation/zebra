use crate::{
    block::{Block, MAX_BLOCK_BYTES},
    serialization::{CompactSizeMessage, ZcashDeserialize, ZcashSerialize},
    work::equihash::{Solution, SOLUTION_SIZE},
};

#[test]
fn equihash_solution_test_vectors() {
    let _init_guard = zebra_test::init();

    // Deserialize each full block (header + transactions), then round-trip
    // just the equihash solution from the parsed header. This sidesteps
    // hard-coded header-length math, which would be wrong for Ycash blocks
    // whose equihash solution is 400 bytes instead of 1344 (see M5.5).
    for block in zebra_test::vectors::BLOCKS.iter() {
        let block =
            Block::zcash_deserialize(&block[..]).expect("block test vector should deserialize");

        let original = block
            .header
            .solution
            .zcash_serialize_to_vec()
            .expect("solution should serialize");

        let roundtrip = Solution::zcash_deserialize(original.as_slice())
            .expect("test vector equihash solution should deserialize")
            .zcash_serialize_to_vec()
            .expect("solution should re-serialize");

        assert_eq!(original, roundtrip);
    }
}

#[test]
fn equihash_solution_test_vectors_are_valid() -> color_eyre::eyre::Result<()> {
    let _init_guard = zebra_test::init();

    for block in zebra_test::vectors::BLOCKS.iter() {
        let block =
            Block::zcash_deserialize(&block[..]).expect("block test vector should deserialize");

        // Ycash blocks at heights >= UPGRADE_YCASH use (N, K) = (192, 7); all
        // other blocks in the Zebra corpus use the common (200, 9). Derive
        // the parameters from the parsed solution's own variant instead of
        // hard-coding one pair for every fixture.
        let (n, k) = block.header.solution.params();
        block.header.solution.check(&block.header, n, k)?;
    }

    Ok(())
}

static EQUIHASH_SIZE_TESTS: &[usize] = &[
    0,
    1,
    SOLUTION_SIZE - 1,
    SOLUTION_SIZE,
    SOLUTION_SIZE + 1,
    (MAX_BLOCK_BYTES - 1) as usize,
    MAX_BLOCK_BYTES as usize,
];

#[test]
fn equihash_solution_size_field() {
    let _init_guard = zebra_test::init();

    for size in EQUIHASH_SIZE_TESTS.iter().copied() {
        let mut data = Vec::new();

        let size: CompactSizeMessage = size
            .try_into()
            .expect("test size fits in MAX_PROTOCOL_MESSAGE_LEN");
        size.zcash_serialize(&mut data)
            .expect("CompactSize should serialize");
        data.resize(data.len() + SOLUTION_SIZE, 0);

        let result = Solution::zcash_deserialize(data.as_slice());
        if size == SOLUTION_SIZE.try_into().unwrap() {
            result.expect("Correct size field in EquihashSolution should deserialize");
        } else {
            result.expect_err("Wrong size field in EquihashSolution should fail on deserialize");
        }
    }
}
