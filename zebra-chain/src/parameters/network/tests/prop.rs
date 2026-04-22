use proptest::prelude::*;

use super::super::Network;
use crate::{
    block::Height,
    parameters::{NetworkUpgrade, TESTNET_MAX_TIME_START_HEIGHT},
};

proptest! {
    /// Check that the mandatory checkpoint is immediately before the Ycash fork activation.
    ///
    /// On Ycash, pre-fork history is byte-identical to Zcash and is checkpointed;
    /// post-fork consensus goes through the semantic verifier. The boundary
    /// between the two is UPGRADE_YCASH activation, so the mandatory checkpoint
    /// is the block immediately before that height.
    #[test]
    fn mandatory_checkpoint_is_immediately_before_ycash(network in any::<Network>()) {
        let _init_guard = zebra_test::init();

        let pre_ycash_activation = NetworkUpgrade::Ycash
            .activation_height(&network)
            .expect("Ycash activation height is set on Mainnet and default Testnet")
            .previous()
            .expect("Ycash activation should be above min height");

        assert_eq!(network.mandatory_checkpoint_height(), pre_ycash_activation);
    }
    #[test]
    /// Asserts that the activation height is correct for the block
    /// maximum time rule on Testnet is correct.
    fn max_block_times_correct_enforcement(height in any::<Height>()) {
        let _init_guard = zebra_test::init();

        assert!(Network::Mainnet.is_max_block_time_enforced(height));
        assert_eq!(Network::new_default_testnet().is_max_block_time_enforced(height), TESTNET_MAX_TIME_START_HEIGHT <= height);
    }
}
