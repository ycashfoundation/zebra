//! Fixed test vectors for the network consensus parameters.

use zcash_protocol::consensus::{self as zp_consensus, NetworkConstants as _, Parameters};

use crate::{
    amount::{Amount, NonNegative},
    block::Height,
    parameters::{
        network::error::ParametersBuilderError,
        subsidy::{
            self, block_subsidy, founders_reward_address, funding_stream_values,
            FundingStreamReceiver,
        },
        testnet::{
            self, ConfiguredActivationHeights, ConfiguredFundingStreamRecipient,
            ConfiguredFundingStreams, ConfiguredLockboxDisbursement, RegtestParameters,
            MAX_NETWORK_NAME_LENGTH, RESERVED_NETWORK_NAMES,
        },
        Network, NetworkUpgrade, MAINNET_ACTIVATION_HEIGHTS, TESTNET_ACTIVATION_HEIGHTS,
    },
};

/// Checks that every method in the `Parameters` impl for `zebra_chain::Network` has the same output
/// as the Ycash-specific reference `Parameters` impls from librustzcash-ycash
/// (`YcashMainNetwork`/`YcashTestNetwork`) on Ycash Mainnet and the default Testnet.
#[test]
fn check_parameters_impl() {
    // Ycash activates through Canopy and never activates NU5+, so the comparison
    // only covers upgrades that are present on both sides.
    fn check_against_reference<P: Parameters>(network: Network, zp_network: P) {
        let zp_network_upgrades = [
            zp_consensus::NetworkUpgrade::Overwinter,
            zp_consensus::NetworkUpgrade::Sapling,
            zp_consensus::NetworkUpgrade::Ycash,
            zp_consensus::NetworkUpgrade::Blossom,
            zp_consensus::NetworkUpgrade::Heartwood,
            zp_consensus::NetworkUpgrade::Canopy,
        ];

        for nu in zp_network_upgrades {
            let activation_height = network
                .activation_height(nu)
                .expect("must have activation height for past network upgrades");

            assert_eq!(
                activation_height,
                zp_network
                    .activation_height(nu)
                    .expect("must have activation height for past network upgrades"),
                "Parameters::activation_heights() outputs must match"
            );

            let activation_height: u32 = activation_height.into();

            for height in (activation_height - 1)..=(activation_height + 1) {
                for nu in zp_network_upgrades {
                    let height = zp_consensus::BlockHeight::from_u32(height);
                    assert_eq!(
                        network.is_nu_active(nu, height),
                        zp_network.is_nu_active(nu, height),
                        "Parameters::is_nu_active() outputs must match",
                    );
                }
            }
        }

        assert_eq!(
            network.coin_type(),
            zp_network.coin_type(),
            "Parameters::coin_type() outputs must match"
        );
        assert_eq!(
            network.hrp_sapling_extended_spending_key(),
            zp_network.hrp_sapling_extended_spending_key(),
            "Parameters::hrp_sapling_extended_spending_key() outputs must match"
        );
        assert_eq!(
            network.hrp_sapling_extended_full_viewing_key(),
            zp_network.hrp_sapling_extended_full_viewing_key(),
            "Parameters::hrp_sapling_extended_full_viewing_key() outputs must match"
        );
        assert_eq!(
            network.hrp_sapling_payment_address(),
            zp_network.hrp_sapling_payment_address(),
            "Parameters::hrp_sapling_payment_address() outputs must match"
        );
        assert_eq!(
            network.b58_pubkey_address_prefix(),
            zp_network.b58_pubkey_address_prefix(),
            "Parameters::b58_pubkey_address_prefix() outputs must match"
        );
        assert_eq!(
            network.b58_script_address_prefix(),
            zp_network.b58_script_address_prefix(),
            "Parameters::b58_script_address_prefix() outputs must match"
        );
    }

    check_against_reference(Network::Mainnet, zp_consensus::YCASH_MAIN_NETWORK);
    check_against_reference(
        Network::new_default_testnet(),
        zp_consensus::YCASH_TEST_NETWORK,
    );
}

/// Checks that `NetworkUpgrade::activation_height()` returns the activation height of the next
/// network upgrade if it doesn't find an activation height for a prior network upgrade, that the
/// `Genesis` upgrade is always at `Height(0)`, and that the default Mainnet/Testnet/Regtest activation
/// heights are what's expected.
#[test]
fn activates_network_upgrades_correctly() {
    let expected_activation_height = 1;
    let network = testnet::Parameters::build()
        .with_activation_heights(ConfiguredActivationHeights {
            nu7: Some(expected_activation_height),
            ..Default::default()
        })
        .expect("failed to set activation heights")
        .clear_funding_streams()
        .to_network()
        .expect("failed to build configured network");

    let genesis_activation_height = NetworkUpgrade::Genesis
        .activation_height(&network)
        .expect("must return an activation height");

    assert_eq!(
        genesis_activation_height,
        Height(0),
        "activation height for all networks after Genesis and BeforeOverwinter should match NU5 activation height"
    );

    for nu in NetworkUpgrade::iter().skip(1) {
        let activation_height = nu
            .activation_height(&network)
            .expect("must return an activation height");

        assert_eq!(
            activation_height, Height(expected_activation_height),
            "activation height for all networks after Genesis and BeforeOverwinter \
            should match NU5 activation height, network_upgrade: {nu}, activation_height: {activation_height:?}"
        );
    }

    let expected_default_regtest_activation_heights = &[
        (Height(0), NetworkUpgrade::Genesis),
        (Height(1), NetworkUpgrade::Canopy),
        (Height(1), NetworkUpgrade::Nu7),
    ];

    for (network, expected_activation_heights) in [
        (Network::Mainnet, MAINNET_ACTIVATION_HEIGHTS),
        (Network::new_default_testnet(), TESTNET_ACTIVATION_HEIGHTS),
        (
            Network::new_regtest(
                ConfiguredActivationHeights {
                    nu7: Some(1),
                    ..Default::default()
                }
                .into(),
            ),
            expected_default_regtest_activation_heights,
        ),
    ] {
        assert_eq!(
            network.activation_list(),
            expected_activation_heights.iter().cloned().collect(),
            "network activation list should match expected activation heights"
        );
    }
}

/// Checks that the founders' reward address displayed prefix switches at
/// `UPGRADE_YCASH` on both Mainnet and the default Testnet, matches the
/// expected Ycash base58 prefixes pre- and post-fork, and vanishes at
/// `YDF_MANDATE_END_HEIGHT`.
///
/// This encodes the address-prefix-at-UPGRADE_YCASH boundary rule. It is a
/// consequence of (a) `founders_reward_address` dispatching on height and
/// selecting different address lists, and (b) `NetworkKind::b58_*_prefix()`
/// emitting Ycash prefixes on output (Milestone 2). No separate consensus
/// enforcement path is required — there is no such rule in ycashd beyond the
/// founders' reward script check itself.
#[test]
fn founders_reward_address_prefix_switches_at_upgrade_ycash() {
    for (network, ydf_end) in [
        (Network::Mainnet, subsidy::constants::mainnet::YDF_MANDATE_END_HEIGHT),
        (
            Network::new_default_testnet(),
            subsidy::constants::testnet::YDF_MANDATE_END_HEIGHT,
        ),
    ] {
        let ycash_activation = NetworkUpgrade::Ycash
            .activation_height(&network)
            .expect("Ycash activates on Mainnet and the default Testnet");

        // Pre-fork: address is selected from the legacy Zcash founders list
        // (P2SH), re-encoded with the network's Ycash base58 script prefix.
        let pre_fork_height = ycash_activation.previous().unwrap();
        let pre_fork_addr = founders_reward_address(&network, pre_fork_height)
            .expect("pre-fork founders' reward address is defined");
        assert!(
            pre_fork_addr.is_script_hash(),
            "pre-UPGRADE_YCASH founders' reward address must be P2SH on {network:?}"
        );
        let (expected_pre_prefix, _) = match network {
            Network::Mainnet => ("s3", "s1"),
            Network::Testnet(_) => ("s2", "sm"),
        };
        assert!(
            pre_fork_addr.to_string().starts_with(expected_pre_prefix),
            "pre-fork address on {network:?} should start with {expected_pre_prefix}, got {pre_fork_addr}"
        );

        // At/after UPGRADE_YCASH (but before YDF mandate end): address comes
        // from YCASH_FOUNDER_ADDRESS_LIST (P2PKH on Ycash) and displays with
        // the Ycash public-key base58 prefix.
        let post_fork_addr = founders_reward_address(&network, ycash_activation)
            .expect("post-fork founders' reward address is defined at UPGRADE_YCASH");
        assert!(
            !post_fork_addr.is_script_hash(),
            "post-UPGRADE_YCASH founders' reward address must be P2PKH on {network:?}"
        );
        let (_, expected_post_prefix) = match network {
            Network::Mainnet => ("s3", "s1"),
            Network::Testnet(_) => ("s2", "sm"),
        };
        assert!(
            post_fork_addr.to_string().starts_with(expected_post_prefix),
            "post-fork address on {network:?} should start with {expected_post_prefix}, got {post_fork_addr}"
        );

        // At YDF mandate end, no founders' reward address is defined.
        assert_eq!(
            founders_reward_address(&network, Height(ydf_end)),
            None,
            "at or after YDF_MANDATE_END_HEIGHT there is no founders' reward on {network:?}"
        );
    }
}

/// Checks that configured testnet names are validated and used correctly.
#[test]
fn check_configured_network_name() {
    // Checks that reserved network names cannot be used for configured testnets.
    for reserved_network_name in RESERVED_NETWORK_NAMES {
        let err = testnet::Parameters::build()
            .with_network_name(reserved_network_name.to_string())
            .expect_err("should fail when using reserved network name");

        assert!(
            matches!(err, ParametersBuilderError::ReservedNetworkName { .. }),
            "unexpected error: {err:?}"
        )
    }

    // Check that max length is enforced
    let err = testnet::Parameters::build()
        .with_network_name("a".repeat(MAX_NETWORK_NAME_LENGTH + 1))
        .expect_err("should fail for invalid name");

    assert!(
        matches!(err, ParametersBuilderError::NetworkNameTooLong { .. }),
        "unexpected error: {err:?}"
    );

    // Check that network names may only contain alphanumeric characters and '_'.
    let err = testnet::Parameters::build()
        .with_network_name("!!!!non-alphanumeric-name".to_string())
        .expect_err("should fail for invalid name");

    assert!(
        matches!(err, ParametersBuilderError::InvalidCharacter),
        "unexpected error: {err:?}"
    );

    // Checks that network names are displayed correctly
    assert_eq!(
        Network::new_default_testnet().to_string(),
        "Testnet",
        "default testnet should be displayed as 'Testnet'"
    );
    assert_eq!(
        Network::Mainnet.to_string(),
        "Mainnet",
        "Mainnet should be displayed as 'Mainnet'"
    );
    assert_eq!(
        Network::new_regtest(Default::default()).to_string(),
        "Regtest",
        "Regtest should be displayed as 'Regtest'"
    );

    // Check that network name can contain alphanumeric characters and '_'.
    let expected_name = "ConfiguredTestnet_1";
    let network = testnet::Parameters::build()
        // Check that network name can contain `MAX_NETWORK_NAME_LENGTH` characters
        .with_network_name("a".repeat(MAX_NETWORK_NAME_LENGTH))
        .expect("failed to set first network name")
        .with_network_name(expected_name)
        .expect("failed to set expected network name")
        .to_network()
        .expect("failed to build configured network");

    // Check that configured network name is displayed
    assert_eq!(
        network.to_string(),
        expected_name,
        "network must be displayed as configured network name"
    );
}

/// Checks that configured testnet names are validated and used correctly.
#[test]
fn check_network_name() {
    // Checks that reserved network names cannot be used for configured testnets.
    for reserved_network_name in RESERVED_NETWORK_NAMES {
        let err = testnet::Parameters::build()
            .with_network_name(reserved_network_name.to_string())
            .expect_err("should fail when using reserved network name");

        assert!(
            matches!(err, ParametersBuilderError::ReservedNetworkName { .. }),
            "unexpected error: {err:?}"
        )
    }

    // Check that max length is enforced
    let err = testnet::Parameters::build()
        .with_network_name("a".repeat(MAX_NETWORK_NAME_LENGTH + 1))
        .expect_err("should fail for invalid name");

    assert!(
        matches!(err, ParametersBuilderError::NetworkNameTooLong { .. }),
        "unexpected error: {err:?}"
    );

    // Check that network names may only contain alphanumeric characters and '_'.
    let err = testnet::Parameters::build()
        .with_network_name("!!!!non-alphanumeric-name".to_string())
        .expect_err("should fail for invalid name");

    assert!(
        matches!(err, ParametersBuilderError::InvalidCharacter),
        "unexpected error: {err:?}"
    );

    // Checks that network names are displayed correctly
    assert_eq!(
        Network::new_default_testnet().to_string(),
        "Testnet",
        "default testnet should be displayed as 'Testnet'"
    );
    assert_eq!(
        Network::Mainnet.to_string(),
        "Mainnet",
        "Mainnet should be displayed as 'Mainnet'"
    );

    // TODO: Check Regtest

    // Check that network name can contain alphanumeric characters and '_'.
    let expected_name = "ConfiguredTestnet_1";
    let network = testnet::Parameters::build()
        // Check that network name can contain `MAX_NETWORK_NAME_LENGTH` characters
        .with_network_name("a".repeat(MAX_NETWORK_NAME_LENGTH))
        .expect("failed to set first network name")
        .with_network_name(expected_name)
        .expect("failed to set expected network name")
        .to_network()
        .expect("failed to build configured network");

    // Check that configured network name is displayed
    assert_eq!(
        network.to_string(),
        expected_name,
        "network must be displayed as configured network name"
    );
}

#[test]
fn check_full_activation_list() {
    let network = testnet::Parameters::build()
        .with_activation_heights(ConfiguredActivationHeights {
            // Update this to be the latest network upgrade in Zebra, and update
            // the code below to expect the latest number of network upgrades.
            nu7: Some(1),
            ..Default::default()
        })
        .expect("failed to set activation heights")
        .clear_funding_streams()
        .to_network()
        .expect("failed to build configured network");

    // We expect the first 11 network upgrades to be included, up to and including NU7
    let expected_network_upgrades = NetworkUpgrade::iter().take(11);
    let full_activation_list_network_upgrades: Vec<_> = network
        .full_activation_list()
        .into_iter()
        .map(|(_, nu)| nu)
        .collect();

    for expected_network_upgrade in expected_network_upgrades {
        assert!(
            full_activation_list_network_upgrades.contains(&expected_network_upgrade),
            "full activation list should contain expected network upgrade"
        );
    }
}

/// Tests that a set of constraints are enforced when building Testnet parameters,
/// and that funding stream configurations that should be valid can be built.
#[test]
fn check_configured_funding_stream_constraints() {
    // On Ycash the default funding-stream table is empty, so the upstream
    // "configured values fall back to defaults" loop doesn't apply. The
    // structural builder-validation panics below are still meaningful; we
    // hand-seed addresses from the Ycash founder lists to ensure each fixture
    // is parseable on the target network.
    std::panic::set_hook(Box::new(|_| {}));

    // should panic when there are fewer addresses than the max funding stream address index.
    let expected_panic_num_addresses = std::panic::catch_unwind(|| {
        testnet::Parameters::build()
            .with_funding_streams(vec![ConfiguredFundingStreams {
                recipients: Some(vec![ConfiguredFundingStreamRecipient {
                    receiver: FundingStreamReceiver::Ecc,
                    numerator: 10,
                    addresses: Some(vec![]),
                }]),
                ..Default::default()
            }])
            .to_network()
    });

    // should panic when sum of numerators is greater than funding stream denominator.
    let expected_panic_numerator = std::panic::catch_unwind(|| {
        testnet::Parameters::build()
            .with_funding_streams(vec![ConfiguredFundingStreams {
                recipients: Some(vec![ConfiguredFundingStreamRecipient {
                    receiver: FundingStreamReceiver::Ecc,
                    numerator: 101,
                    addresses: Some(
                        subsidy::constants::testnet::YCASH_FOUNDER_ADDRESS_LIST
                            .iter()
                            .map(|s| s.to_string())
                            .collect(),
                    ),
                }]),
                ..Default::default()
            }])
            .to_network()
    });

    // should panic when recipient addresses are for Mainnet.
    let expected_panic_wrong_addr_network = std::panic::catch_unwind(|| {
        testnet::Parameters::build()
            .with_funding_streams(vec![ConfiguredFundingStreams {
                recipients: Some(vec![ConfiguredFundingStreamRecipient {
                    receiver: FundingStreamReceiver::Ecc,
                    numerator: 10,
                    addresses: Some(
                        subsidy::constants::mainnet::YCASH_FOUNDER_ADDRESS_LIST
                            .iter()
                            .map(|s| s.to_string())
                            .collect(),
                    ),
                }]),
                ..Default::default()
            }])
            .to_network()
    });

    // drop panic hook before expecting errors.
    let _ = std::panic::take_hook();

    expected_panic_num_addresses.expect_err("should panic when there are too few addresses");
    expected_panic_numerator.expect_err(
        "should panic when sum of numerators is greater than funding stream denominator",
    );
    expected_panic_wrong_addr_network
        .expect_err("should panic when recipient addresses are for Mainnet");
}

/// Check that `new_regtest()` constructs a network with the provided funding streams.
#[test]
fn check_configured_funding_stream_regtest() {
    // On Ycash the default testnet has no funding streams, so the upstream
    // version's "derive a regtest fixture from default_testnet" pattern
    // doesn't work here. Hand-construct two minimal regtest fixtures — the
    // point of the test is that `Network::new_regtest()` preserves the
    // configured funding streams verbatim. We use the Ycash testnet founder
    // list to guarantee the addresses parse on the target network.
    let default_testnet = Network::new_default_testnet();

    let addresses: Vec<String> = subsidy::constants::testnet::YCASH_FOUNDER_ADDRESS_LIST
        .iter()
        .map(|s| s.to_string())
        .collect();

    let configured_pre_nu6_funding_streams = ConfiguredFundingStreams {
        height_range: Some(Height(100)..Height(120)),
        recipients: Some(vec![ConfiguredFundingStreamRecipient {
            receiver: FundingStreamReceiver::MajorGrants,
            numerator: 8,
            addresses: Some(addresses.clone()),
        }]),
    };

    let configured_post_nu6_funding_streams = ConfiguredFundingStreams {
        height_range: Some(Height(200)..Height(220)),
        recipients: Some(vec![
            ConfiguredFundingStreamRecipient {
                receiver: FundingStreamReceiver::Deferred,
                numerator: 12,
                addresses: None,
            },
            ConfiguredFundingStreamRecipient {
                receiver: FundingStreamReceiver::MajorGrants,
                numerator: 8,
                addresses: Some(addresses),
            },
        ]),
    };

    let regtest = Network::new_regtest(RegtestParameters {
        activation_heights: (&default_testnet.activation_list()).into(),
        funding_streams: Some(vec![
            configured_pre_nu6_funding_streams.clone(),
            configured_post_nu6_funding_streams.clone(),
        ]),
        ..Default::default()
    });

    let expected_pre_nu6_funding_streams =
        configured_pre_nu6_funding_streams.into_funding_streams_unchecked();
    let expected_post_nu6_funding_streams =
        configured_post_nu6_funding_streams.into_funding_streams_unchecked();

    assert_eq!(
        &expected_pre_nu6_funding_streams,
        &regtest.all_funding_streams()[0]
    );
    assert_eq!(
        &expected_post_nu6_funding_streams,
        &regtest.all_funding_streams()[1]
    );
}

// ZIP-271 one-time lockbox disbursement machinery is NU6.1-specific. Ycash
// never activates NU6.1 on Mainnet or the default Testnet, and the helper
// `lockbox_input_value` below asserts that the `Deferred` funding stream
// exists (which is tied to the Zcash post-Canopy funding-stream table that
// Ycash has replaced with a continuous founders' reward). Re-enable and
// adapt if Ycash ever ships NU6.1.
#[test]
#[ignore = "ZIP-271 lockbox disbursements don't apply on Ycash"]
fn sum_of_one_time_lockbox_disbursements_is_correct() {
    let mut configured_activation_heights: ConfiguredActivationHeights =
        Network::new_default_testnet().activation_list().into();
    configured_activation_heights.nu6_1 = Some(2_976_000 + 420_000);

    let custom_testnet = testnet::Parameters::build()
        .with_activation_heights(configured_activation_heights)
        .expect("failed to set activation heights")
        .with_lockbox_disbursements(vec![ConfiguredLockboxDisbursement {
            address: "t26ovBdKAJLtrvBsE2QGF4nqBkEuptuPFZz".to_string(),
            amount: Amount::new_from_zec(78_750),
        }])
        .to_network()
        .expect("failed to build configured network");

    for network in Network::iter().chain(std::iter::once(custom_testnet)) {
        let Some(nu6_1_activation_height) = NetworkUpgrade::Nu6_1.activation_height(&network)
        else {
            tracing::warn!(
                ?network,
                "skipping check as there's no NU6.1 activation height for this network"
            );
            continue;
        };

        let total_disbursement_output_value = network
            .lockbox_disbursements(nu6_1_activation_height)
            .into_iter()
            .map(|(_addr, expected_amount)| expected_amount)
            .try_fold(crate::amount::Amount::zero(), |a, b| a + b)
            .expect("sum of output values should be valid Amount");

        assert_eq!(
            total_disbursement_output_value,
            network.lockbox_disbursement_total_amount(nu6_1_activation_height),
            "sum of lockbox disbursement output values should match expected total"
        );

        let last_nu6_height = nu6_1_activation_height.previous().unwrap();
        let expected_total_lockbox_disbursement_value =
            lockbox_input_value(&network, last_nu6_height);

        assert_eq!(
            expected_total_lockbox_disbursement_value,
            network.lockbox_disbursement_total_amount(nu6_1_activation_height),
            "total lockbox disbursement value should match expected total"
        );
    }
}

/// Lockbox funding stream total input value for a block height.
///
/// Assumes a constant funding stream amount per block.
fn lockbox_input_value(network: &Network, height: Height) -> Amount<NonNegative> {
    let Some(nu6_activation_height) = NetworkUpgrade::Nu6.activation_height(network) else {
        return Amount::zero();
    };

    let total_block_subsidy = block_subsidy(height, network).unwrap();
    let &deferred_amount_per_block =
        funding_stream_values(nu6_activation_height, network, total_block_subsidy)
            .expect("we always expect a funding stream hashmap response even if empty")
            .get(&FundingStreamReceiver::Deferred)
            .expect("we expect a lockbox funding stream after NU5");

    let post_nu6_funding_stream_height_range = network.all_funding_streams()[1].height_range();

    // `min(height, last_height_with_deferred_pool_contribution) - (nu6_activation_height - 1)`,
    // We decrement NU6 activation height since it's an inclusive lower bound.
    // Funding stream height range end bound is not incremented since it's an exclusive end bound
    let num_blocks_with_lockbox_output = (height.0 + 1)
        .min(post_nu6_funding_stream_height_range.end.0)
        .saturating_sub(post_nu6_funding_stream_height_range.start.0);

    (deferred_amount_per_block * num_blocks_with_lockbox_output.into())
        .expect("lockbox input value should fit in Amount")
}

#[test]
fn funding_streams_default_values() {
    let _init_guard = zebra_test::init();

    // Upstream Zebra tested that the testnet builder fills in defaults for
    // unset fields of a `ConfiguredFundingStreams`. On Ycash the default
    // `testnet::FUNDING_STREAMS` table is empty (Ycash has no funding streams
    // — see `founders_reward` for the replacement), so the
    // "unconfigured-recipients-fall-back-to-defaults" path has nothing to
    // check. Verify instead that the default table really is empty and that a
    // fully-configured stream is preserved verbatim.
    assert!(
        subsidy::constants::testnet::FUNDING_STREAMS.is_empty(),
        "Ycash testnet default FUNDING_STREAMS should be empty"
    );
    assert!(
        subsidy::constants::mainnet::FUNDING_STREAMS.is_empty(),
        "Ycash mainnet default FUNDING_STREAMS should be empty"
    );

    // Use a short height range so the 48-address YCASH_FOUNDER_ADDRESS_LIST
    // is more than enough to satisfy `check_funding_stream_address_period`.
    let fs = vec![ConfiguredFundingStreams {
        height_range: Some(Height(1_028_500)..Height(1_028_600)),
        recipients: Some(vec![
            ConfiguredFundingStreamRecipient {
                receiver: FundingStreamReceiver::Deferred,
                numerator: 1,
                addresses: None,
            },
            ConfiguredFundingStreamRecipient {
                receiver: FundingStreamReceiver::MajorGrants,
                numerator: 2,
                addresses: Some(
                    subsidy::constants::testnet::YCASH_FOUNDER_ADDRESS_LIST
                        .iter()
                        .map(|s| s.to_string())
                        .collect(),
                ),
            },
        ]),
    }];

    let network = testnet::Parameters::build()
        .with_funding_streams(fs)
        .to_network()
        .expect("failed to build configured network");

    assert_eq!(
        network.all_funding_streams()[0].height_range().clone(),
        Height(1_028_500)..Height(1_028_600)
    );
    assert_eq!(
        network.all_funding_streams()[0]
            .recipients()
            .get(&FundingStreamReceiver::Deferred)
            .unwrap()
            .numerator(),
        1
    );
    assert_eq!(
        network.all_funding_streams()[0]
            .recipients()
            .get(&FundingStreamReceiver::MajorGrants)
            .unwrap()
            .numerator(),
        2
    );
}
