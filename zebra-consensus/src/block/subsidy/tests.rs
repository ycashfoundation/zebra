//! Tests for funding streams.

#![allow(clippy::unwrap_in_result)]

use color_eyre::Report;
use zebra_chain::parameters::NetworkUpgrade;

use super::*;

/// Checks that Ycash has no funding streams on any network.
///
/// Zcash swapped founders' reward -> dev/ECC/ZF/MG funding streams at Canopy
/// activation; Ycash does not, continuing founders' reward through
/// `nYdfMandateEndHeight`. The Ycash FUNDING_STREAMS arrays are therefore
/// empty (M3 facdbde2), and `funding_stream_values()` should always return an
/// empty map regardless of height.
#[test]
fn ycash_has_no_funding_streams() -> Result<(), Report> {
    let _init_guard = zebra_test::init();

    for network in Network::iter() {
        assert!(
            network.all_funding_streams().is_empty(),
            "Ycash {network:?} should have no funding streams, got {streams:?}",
            streams = network.all_funding_streams()
        );

        // Sample a range of heights that would correspond to Zcash's funding-stream
        // period (around Canopy, plus later heights where Zcash NU6/NU6.1 would start),
        // and confirm funding_stream_values() is empty at every one.
        let canopy_activation_height = NetworkUpgrade::Canopy
            .activation_height(&network)
            .expect("Canopy has an activation height on Ycash Mainnet/Testnet");
        let sample_heights = [
            Height(1),
            canopy_activation_height.previous().unwrap(),
            canopy_activation_height,
            canopy_activation_height.next().unwrap(),
            Height(2_000_000),
            Height(3_000_000),
            Height(4_000_000),
        ];
        for height in sample_heights {
            let fsv = funding_stream_values(height, &network, block_subsidy(height, &network)?)
                .expect("funding_stream_values is Ok when streams are empty");
            assert!(
                fsv.is_empty(),
                "funding_stream_values at {height:?} on {network:?} should be empty, got {fsv:?}"
            );
        }
    }

    Ok(())
}

/// Check mainnet and testnet funding stream addresses are valid transparent P2SH addresses.
///
/// Ycash has no funding streams, so this loop is vacuous on real networks but
/// still exercises the iteration/branching over `all_funding_streams()`.
#[test]
fn test_funding_stream_addresses() -> Result<(), Report> {
    let _init_guard = zebra_test::init();
    for network in Network::iter() {
        for (receiver, recipient) in network
            .all_funding_streams()
            .iter()
            .flat_map(|fs| fs.recipients())
        {
            for address in recipient.addresses() {
                let expected_network_kind = match network.kind() {
                    zebra_chain::parameters::NetworkKind::Mainnet => {
                        zebra_chain::parameters::NetworkKind::Mainnet
                    }
                    // `Regtest` uses `Testnet` transparent addresses.
                    zebra_chain::parameters::NetworkKind::Testnet
                    | zebra_chain::parameters::NetworkKind::Regtest => {
                        zebra_chain::parameters::NetworkKind::Testnet
                    }
                };

                assert_eq!(
                    address.network_kind(),
                    expected_network_kind,
                    "incorrect network for {receiver:?} funding stream address constant: {address}",
                );

                assert!(
                    address.is_script_hash(),
                    "funding stream address is not P2SH: {address}"
                );

                let _script = address.script();
            }
        }
    }

    Ok(())
}

//Test if funding streams ranges do not overlap
#[test]
fn test_funding_stream_ranges_dont_overlap() -> Result<(), Report> {
    let _init_guard = zebra_test::init();
    for network in Network::iter() {
        let funding_streams = network.all_funding_streams();
        // This is quadratic but it's fine since the number of funding streams is small.
        for i in 0..funding_streams.len() {
            for j in (i + 1)..funding_streams.len() {
                let range_a = funding_streams[i].height_range();
                let range_b = funding_streams[j].height_range();
                assert!(
                    // https://stackoverflow.com/a/325964
                    !(range_a.start < range_b.end && range_b.start < range_a.end),
                    "Funding streams {i} and {j} overlap: {range_a:?} and {range_b:?}",
                );
            }
        }
    }
    Ok(())
}
