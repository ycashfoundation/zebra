//! Calculations for Block Subsidy and Funding Streams
//!
//! This module contains the consensus parameters which are required for
//! verification.
//!
//! Some consensus parameters change based on network upgrades. Each network
//! upgrade happens at a particular block height. Some parameters have a value
//! (or function) before the upgrade height, at the upgrade height, and after
//! the upgrade height. (For example, the value of the reserved field in the
//! block header during the Heartwood upgrade.)
//!
//! Typically, consensus parameters are accessed via a function that takes a
//! `Network` and `block::Height`.

pub(crate) mod constants;

use std::collections::HashMap;

use crate::{
    amount::{self, Amount, NonNegative},
    block::{Height, HeightDiff},
    parameters::{Network, NetworkUpgrade},
    transparent,
};

use constants::{
    regtest, BLOSSOM_POW_TARGET_SPACING_RATIO, FUNDING_STREAM_RECEIVER_DENOMINATOR,
    FUNDING_STREAM_SPECIFICATION, LOCKBOX_SPECIFICATION, MAX_BLOCK_SUBSIDY,
    POST_BLOSSOM_HALVING_INTERVAL, PRE_BLOSSOM_HALVING_INTERVAL,
};

/// The funding stream receiver categories.
#[derive(Serialize, Deserialize, Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum FundingStreamReceiver {
    /// The Electric Coin Company (Bootstrap Foundation) funding stream.
    #[serde(rename = "ECC")]
    Ecc,

    /// The Zcash Foundation funding stream.
    ZcashFoundation,

    /// The Major Grants (Zcash Community Grants) funding stream.
    MajorGrants,

    /// The deferred pool contribution, see [ZIP-1015](https://zips.z.cash/zip-1015) for more details.
    Deferred,
}

impl FundingStreamReceiver {
    /// Returns a human-readable name and a specification URL for the receiver, as described in
    /// [ZIP-1014] and [`zcashd`] before NU6. After NU6, the specification is in the [ZIP-1015].
    ///
    /// [ZIP-1014]: https://zips.z.cash/zip-1014#abstract
    /// [`zcashd`]: https://github.com/zcash/zcash/blob/3f09cfa00a3c90336580a127e0096d99e25a38d6/src/consensus/funding.cpp#L13-L32
    /// [ZIP-1015]: https://zips.z.cash/zip-1015
    pub fn info(&self, is_post_nu6: bool) -> (&'static str, &'static str) {
        if is_post_nu6 {
            (
                match self {
                    FundingStreamReceiver::Ecc => "Electric Coin Company",
                    FundingStreamReceiver::ZcashFoundation => "Zcash Foundation",
                    FundingStreamReceiver::MajorGrants => "Zcash Community Grants NU6",
                    FundingStreamReceiver::Deferred => "Lockbox NU6",
                },
                LOCKBOX_SPECIFICATION,
            )
        } else {
            (
                match self {
                    FundingStreamReceiver::Ecc => "Electric Coin Company",
                    FundingStreamReceiver::ZcashFoundation => "Zcash Foundation",
                    FundingStreamReceiver::MajorGrants => "Major Grants",
                    FundingStreamReceiver::Deferred => "Lockbox NU6",
                },
                FUNDING_STREAM_SPECIFICATION,
            )
        }
    }

    /// Returns true if this [`FundingStreamReceiver`] is [`FundingStreamReceiver::Deferred`].
    pub fn is_deferred(&self) -> bool {
        matches!(self, Self::Deferred)
    }
}

/// Funding stream recipients and height ranges.
#[derive(Deserialize, Clone, Debug, Eq, PartialEq)]
pub struct FundingStreams {
    /// Start and end Heights for funding streams
    /// as described in [protocol specification §7.10.1][7.10.1].
    ///
    /// [7.10.1]: https://zips.z.cash/protocol/protocol.pdf#zip214fundingstreams
    height_range: std::ops::Range<Height>,
    /// Funding stream recipients by [`FundingStreamReceiver`].
    recipients: HashMap<FundingStreamReceiver, FundingStreamRecipient>,
}

impl FundingStreams {
    /// Creates a new [`FundingStreams`].
    pub fn new(
        height_range: std::ops::Range<Height>,
        recipients: HashMap<FundingStreamReceiver, FundingStreamRecipient>,
    ) -> Self {
        Self {
            height_range,
            recipients,
        }
    }

    /// Creates a new empty [`FundingStreams`] representing no funding streams.
    pub fn empty() -> Self {
        Self::new(Height::MAX..Height::MAX, HashMap::new())
    }

    /// Returns height range where these [`FundingStreams`] should apply.
    pub fn height_range(&self) -> &std::ops::Range<Height> {
        &self.height_range
    }

    /// Returns recipients of these [`FundingStreams`].
    pub fn recipients(&self) -> &HashMap<FundingStreamReceiver, FundingStreamRecipient> {
        &self.recipients
    }

    /// Returns a recipient with the provided receiver.
    pub fn recipient(&self, receiver: FundingStreamReceiver) -> Option<&FundingStreamRecipient> {
        self.recipients.get(&receiver)
    }

    /// Accepts a target number of addresses that all recipients of this funding stream
    /// except the [`FundingStreamReceiver::Deferred`] receiver should have.
    ///
    /// Extends the addresses for all funding stream recipients by repeating their
    /// existing addresses until reaching the provided target number of addresses.
    pub fn extend_recipient_addresses(&mut self, target_len: usize) {
        for (receiver, recipient) in &mut self.recipients {
            if receiver.is_deferred() {
                continue;
            }

            recipient.extend_addresses(target_len);
        }
    }
}

/// A funding stream recipient as specified in [protocol specification §7.10.1][7.10.1]
///
/// [7.10.1]: https://zips.z.cash/protocol/protocol.pdf#zip214fundingstreams
#[derive(Deserialize, Clone, Debug, Eq, PartialEq)]
pub struct FundingStreamRecipient {
    /// The numerator for each funding stream receiver category
    /// as described in [protocol specification §7.10.1][7.10.1].
    ///
    /// [7.10.1]: https://zips.z.cash/protocol/protocol.pdf#zip214fundingstreams
    numerator: u64,
    /// Addresses for the funding stream recipient
    addresses: Vec<transparent::Address>,
}

impl FundingStreamRecipient {
    /// Creates a new [`FundingStreamRecipient`].
    pub fn new<I, T>(numerator: u64, addresses: I) -> Self
    where
        T: ToString,
        I: IntoIterator<Item = T>,
    {
        Self {
            numerator,
            addresses: addresses
                .into_iter()
                .map(|addr| {
                    let addr = addr.to_string();
                    addr.parse()
                        .expect("funding stream address must deserialize")
                })
                .collect(),
        }
    }

    /// Returns the numerator for this funding stream.
    pub fn numerator(&self) -> u64 {
        self.numerator
    }

    /// Returns the receiver of this funding stream.
    pub fn addresses(&self) -> &[transparent::Address] {
        &self.addresses
    }

    /// Accepts a target number of addresses that this recipient should have.
    ///
    /// Extends the addresses for this funding stream recipient by repeating
    /// existing addresses until reaching the provided target number of addresses.
    ///
    /// # Panics
    ///
    /// If there are no recipient addresses.
    pub fn extend_addresses(&mut self, target_len: usize) {
        assert!(
            !self.addresses.is_empty(),
            "cannot extend addresses for empty recipient"
        );

        self.addresses = self
            .addresses
            .iter()
            .cycle()
            .take(target_len)
            .cloned()
            .collect();
    }
}

/// Functionality specific to block subsidy-related consensus rules
pub trait ParameterSubsidy {
    /// Returns the minimum height after the first halving
    /// as described in [protocol specification §7.10][7.10]
    ///
    /// [7.10]: <https://zips.z.cash/protocol/protocol.pdf#fundingstreams>
    fn height_for_first_halving(&self) -> Height;

    /// Returns the halving interval after Blossom
    fn post_blossom_halving_interval(&self) -> HeightDiff;

    /// Returns the halving interval before Blossom
    fn pre_blossom_halving_interval(&self) -> HeightDiff;

    /// Returns the address change interval for funding streams
    /// as described in [protocol specification §7.10][7.10].
    ///
    /// > FSRecipientChangeInterval := PostBlossomHalvingInterval / 48
    ///
    /// [7.10]: https://zips.z.cash/protocol/protocol.pdf#zip214fundingstreams
    fn funding_stream_address_change_interval(&self) -> HeightDiff;
}

/// Network methods related to Block Subsidy and Funding Streams
impl ParameterSubsidy for Network {
    fn height_for_first_halving(&self) -> Height {
        // Compute the first halving height from the halving formula on all networks
        // except Regtest (which pins a fixed small value for test convenience).
        //
        // Upstream Zebra hardcoded Mainnet → Canopy activation and default Testnet →
        // `testnet::FIRST_HALVING`; both are Zcash-specific coincidences of Zcash's
        // activation schedule where Blossom precedes the first halving. On Ycash
        // mainnet the first halving (850_000) is pre-Blossom, so neither hardcode is
        // correct here.
        match self {
            Network::Testnet(params) if params.is_regtest() => regtest::FIRST_HALVING,
            _ => height_for_halving(1, self).expect("first halving height should be available"),
        }
    }

    fn post_blossom_halving_interval(&self) -> HeightDiff {
        match self {
            Network::Mainnet => POST_BLOSSOM_HALVING_INTERVAL,
            Network::Testnet(params) => params.post_blossom_halving_interval(),
        }
    }

    fn pre_blossom_halving_interval(&self) -> HeightDiff {
        match self {
            Network::Mainnet => PRE_BLOSSOM_HALVING_INTERVAL,
            Network::Testnet(params) => params.pre_blossom_halving_interval(),
        }
    }

    fn funding_stream_address_change_interval(&self) -> HeightDiff {
        self.post_blossom_halving_interval() / 48
    }
}

/// Returns the address change period
/// as described in [protocol specification §7.10][7.10]
///
/// [7.10]: https://zips.z.cash/protocol/protocol.pdf#fundingstreams
pub fn funding_stream_address_period<N: ParameterSubsidy>(height: Height, network: &N) -> u32 {
    // Spec equation: `address_period = floor((height - (height_for_halving(1) - post_blossom_halving_interval))/funding_stream_address_change_interval)`,
    // <https://zips.z.cash/protocol/protocol.pdf#fundingstreams>
    //
    // Note that the brackets make it so the post blossom halving interval is added to the total.
    //
    // In Rust, "integer division rounds towards zero":
    // <https://doc.rust-lang.org/stable/reference/expressions/operator-expr.html#arithmetic-and-logical-binary-operators>
    // This is the same as `floor()`, because these numbers are all positive.

    let height_after_first_halving = height - network.height_for_first_halving();

    let address_period = (height_after_first_halving + network.post_blossom_halving_interval())
        / network.funding_stream_address_change_interval();

    address_period
        .try_into()
        .expect("all values are positive and smaller than the input height")
}

/// The first block height of the halving at the provided halving index for a network.
///
/// See `HalvingHeight(height, halvingIndex)` as described in ZIP-208 and
/// [protocol specification §7.8][7.8].
///
/// Uses the pre-Blossom branch when the halving index's pre-Blossom candidate
/// height precedes Blossom activation (as on Ycash mainnet, where the first
/// halving at 850_000 precedes Blossom at 1_100_000), and the post-Blossom
/// branch otherwise. On Zcash-style networks where Blossom precedes the first
/// halving, every halving falls under the post-Blossom branch.
///
/// [7.8]: https://zips.z.cash/protocol/protocol.pdf#subsidies
pub fn height_for_halving(halving: u32, network: &Network) -> Option<Height> {
    if halving == 0 {
        return Some(Height(0));
    }

    let slow_start_shift = i64::from(network.slow_start_shift().0);
    let blossom_height = i64::from(NetworkUpgrade::Blossom.activation_height(network)?.0);
    let pre_blossom_halving_interval = network.pre_blossom_halving_interval();
    let post_blossom_halving_interval = network.post_blossom_halving_interval();
    let halving_index = i64::from(halving);

    // ZIP-208 HalvingHeight pre-Blossom branch:
    //   height = preInterval * halvingIndex + slowStartShift
    let pre_blossom_candidate_height = halving_index
        .checked_mul(pre_blossom_halving_interval)?
        .checked_add(slow_start_shift)?;

    let height = if pre_blossom_candidate_height < blossom_height {
        pre_blossom_candidate_height
    } else {
        // ZIP-208 HalvingHeight post-Blossom branch:
        //   height = postInterval * halvingIndex
        //          - BLOSSOM_POW_TARGET_SPACING_RATIO * (blossomHeight - slowStartShift)
        //          + blossomHeight
        let ratio_times_blossom_gap = i64::from(BLOSSOM_POW_TARGET_SPACING_RATIO)
            .checked_mul(blossom_height.checked_sub(slow_start_shift)?)?;

        halving_index
            .checked_mul(post_blossom_halving_interval)?
            .checked_sub(ratio_times_blossom_gap)?
            .checked_add(blossom_height)?
    };

    let height = u32::try_from(height).ok()?;
    height.try_into().ok()
}

/// Returns the `fs.Value(height)` for each stream receiver
/// as described in [protocol specification §7.8][7.8]
///
/// [7.8]: https://zips.z.cash/protocol/protocol.pdf#subsidies
pub fn funding_stream_values(
    height: Height,
    network: &Network,
    expected_block_subsidy: Amount<NonNegative>,
) -> Result<HashMap<FundingStreamReceiver, Amount<NonNegative>>, amount::Error> {
    let mut results = HashMap::new();

    if expected_block_subsidy.is_zero() {
        return Ok(results);
    }

    if NetworkUpgrade::current(network, height) >= NetworkUpgrade::Canopy {
        let funding_streams = network.funding_streams(height);
        if let Some(funding_streams) = funding_streams {
            for (&receiver, recipient) in funding_streams.recipients() {
                // - Spec equation: `fs.value = floor(block_subsidy(height)*(fs.numerator/fs.denominator))`:
                //   https://zips.z.cash/protocol/protocol.pdf#subsidies
                // - In Rust, "integer division rounds towards zero":
                //   https://doc.rust-lang.org/stable/reference/expressions/operator-expr.html#arithmetic-and-logical-binary-operators
                //   This is the same as `floor()`, because these numbers are all positive.
                let amount_value = ((expected_block_subsidy * recipient.numerator())?
                    / FUNDING_STREAM_RECEIVER_DENOMINATOR)?;

                results.insert(receiver, amount_value);
            }
        }
    }

    Ok(results)
}

/// Block subsidy errors.
#[derive(thiserror::Error, Clone, Debug, PartialEq, Eq)]
#[allow(missing_docs)]
pub enum SubsidyError {
    #[error("no coinbase transaction in block")]
    NoCoinbase,

    #[error("funding stream expected output not found")]
    FundingStreamNotFound,

    #[error("founders reward output not found")]
    FoundersRewardNotFound,

    #[error("one-time lockbox disbursement output not found")]
    OneTimeLockboxDisbursementNotFound,

    #[error("miner fees are invalid")]
    InvalidMinerFees,

    #[error("addition of amounts overflowed")]
    Overflow,

    #[error("subtraction of amounts underflowed")]
    Underflow,

    #[error("unsupported height")]
    UnsupportedHeight,

    #[error("invalid amount")]
    InvalidAmount(#[from] amount::Error),

    #[cfg(zcash_unstable = "zip235")]
    #[error("invalid zip233 amount")]
    InvalidZip233Amount,
}

/// The divisor used for halvings.
///
/// `1 << Halving(height)`, as described in [protocol specification §7.8][7.8]
///
/// [7.8]: https://zips.z.cash/protocol/protocol.pdf#subsidies
///
/// Returns `None` if the divisor would overflow a `u64`.
pub fn halving_divisor(height: Height, network: &Network) -> Option<u64> {
    // Some far-future shifts can be more than 63 bits
    1u64.checked_shl(halving(height, network))
}

/// The halving index for a block height and network.
///
/// `Halving(height)`, as described in [protocol specification §7.8][7.8]
///
/// [7.8]: https://zips.z.cash/protocol/protocol.pdf#subsidies
pub fn halving(height: Height, network: &Network) -> u32 {
    let slow_start_shift = network.slow_start_shift();
    let blossom_height = NetworkUpgrade::Blossom
        .activation_height(network)
        .expect("blossom activation height should be available");

    let halving_index = if height < slow_start_shift {
        0
    } else if height < blossom_height {
        let pre_blossom_height = height - slow_start_shift;
        pre_blossom_height / network.pre_blossom_halving_interval()
    } else {
        let pre_blossom_height = blossom_height - slow_start_shift;
        let scaled_pre_blossom_height =
            pre_blossom_height * HeightDiff::from(BLOSSOM_POW_TARGET_SPACING_RATIO);

        let post_blossom_height = height - blossom_height;

        (scaled_pre_blossom_height + post_blossom_height) / network.post_blossom_halving_interval()
    };

    halving_index
        .try_into()
        .expect("already checked for negatives")
}

/// `BlockSubsidy(height)` as described in [protocol specification §7.8][7.8]
///
/// [7.8]: https://zips.z.cash/protocol/protocol.pdf#subsidies
pub fn block_subsidy(height: Height, net: &Network) -> Result<Amount<NonNegative>, SubsidyError> {
    let Some(halving_div) = halving_divisor(height, net) else {
        return Ok(Amount::zero());
    };

    let slow_start_interval = net.slow_start_interval();

    // The `floor` fn used in the spec is implicit in Rust's division of primitive integer types.

    let amount = if height < slow_start_interval {
        let slow_start_rate = MAX_BLOCK_SUBSIDY / u64::from(slow_start_interval);

        if height < net.slow_start_shift() {
            slow_start_rate * u64::from(height)
        } else {
            slow_start_rate * (u64::from(height) + 1)
        }
    } else {
        let base_subsidy = if NetworkUpgrade::current(net, height) < NetworkUpgrade::Blossom {
            MAX_BLOCK_SUBSIDY
        } else {
            MAX_BLOCK_SUBSIDY / u64::from(BLOSSOM_POW_TARGET_SPACING_RATIO)
        };

        base_subsidy / halving_div
    };

    Ok(Amount::try_from(amount)?)
}

/// `MinerSubsidy(height)` as described in [protocol specification §7.8][7.8]
///
/// [7.8]: https://zips.z.cash/protocol/protocol.pdf#subsidies
pub fn miner_subsidy(
    height: Height,
    network: &Network,
    expected_block_subsidy: Amount<NonNegative>,
) -> Result<Amount<NonNegative>, amount::Error> {
    let founders_reward = founders_reward(network, height);

    let funding_streams_sum = funding_stream_values(height, network, expected_block_subsidy)?
        .values()
        .sum::<Result<Amount<NonNegative>, _>>()?;

    expected_block_subsidy - founders_reward - funding_streams_sum
}

/// Returns the founders reward address for a given height and network.
///
/// Implements the two-phase Ycash rule from
/// `ycashd/src/chainparams.cpp::GetFoundersRewardAddressAtHeight`:
///
/// - Before UPGRADE_YCASH: selects from the 48 pre-fork addresses (the legacy
///   Zcash founder list) using the Zcash pre-Blossom / Blossom-adjusted
///   formula, per [§7.9] of the Zcash protocol spec. ycashd's
///   `keyIO.ZecToYec()` re-encoding happens naturally here because Zebra's
///   `NetworkKind::b58_*_address_prefix()` returns Ycash prefixes on output.
/// - At or after UPGRADE_YCASH, and before [`ydf_mandate_end_height`]: selects
///   from the 48 Ycash post-fork addresses using a fixed 17_917-block change
///   interval with modulo wrapping, so the list cycles for the full mandate
///   window.
/// - After the mandate ends, returns `None` (there is no founders' reward to
///   pay).
///
/// [§7.9]: <https://zips.z.cash/protocol/protocol.pdf#foundersreward>
pub fn founders_reward_address(net: &Network, height: Height) -> Option<transparent::Address> {
    let ycash_activation_height = NetworkUpgrade::Ycash.activation_height(net)?;

    if height < ycash_activation_height {
        founders_reward_address_pre_ycash(net, height)
    } else if height < net.ydf_mandate_end_height() {
        founders_reward_address_post_ycash(net, height, ycash_activation_height)
    } else {
        None
    }
}

/// Pre-UPGRADE_YCASH founders' reward address, using the legacy Zcash address
/// list and the pre-Blossom / Blossom-adjusted formula.
fn founders_reward_address_pre_ycash(
    net: &Network,
    height: Height,
) -> Option<transparent::Address> {
    let founders_address_list = net.founder_address_list();
    let num_founder_addresses = u32::try_from(founders_address_list.len()).ok()?;
    let slow_start_shift = u32::from(net.slow_start_shift());
    let pre_blossom_halving_interval = u32::try_from(net.pre_blossom_halving_interval()).ok()?;

    let founder_address_change_interval = slow_start_shift
        .checked_add(pre_blossom_halving_interval)?
        .div_ceil(num_founder_addresses);

    let founder_address_adjusted_height =
        if NetworkUpgrade::current(net, height) < NetworkUpgrade::Blossom {
            u32::from(height)
        } else {
            NetworkUpgrade::Blossom
                .activation_height(net)
                .and_then(|h| {
                    let blossom_activation_height = u32::from(h);
                    let height = u32::from(height);

                    blossom_activation_height.checked_add(
                        height.checked_sub(blossom_activation_height)?
                            / BLOSSOM_POW_TARGET_SPACING_RATIO,
                    )
                })?
        };

    let founder_address_index =
        usize::try_from(founder_address_adjusted_height / founder_address_change_interval).ok()?;

    founders_address_list
        .get(founder_address_index)
        .and_then(|a| a.parse().ok())
}

/// Post-UPGRADE_YCASH founders' reward address, using the Ycash founder list
/// with a fixed change interval and modulo wrapping.
fn founders_reward_address_post_ycash(
    net: &Network,
    height: Height,
    ycash_activation_height: Height,
) -> Option<transparent::Address> {
    let ycash_addresses = net.ycash_founder_address_list();
    if ycash_addresses.is_empty() {
        return None;
    }

    let height = u32::from(height);
    let ycash_activation_height = u32::from(ycash_activation_height);
    let offset = height.checked_sub(ycash_activation_height)?;
    let interval = net.ycash_founder_address_change_interval();

    let index = (offset / interval) as usize % ycash_addresses.len();
    ycash_addresses.get(index).and_then(|a| a.parse().ok())
}

/// `FoundersReward(height)` for Ycash.
///
/// Matches the two-phase rule in `ycashd/src/main.cpp` (search
/// `UPGRADE_YCASH`):
///
/// - Pre-UPGRADE_YCASH: 20% of the block subsidy (`subsidy / 5`), the same as
///   Zcash's founders' reward up to the first halving. ycashd further gates
///   this on `height <= GetLastFoundersRewardBlockHeight(height)`, i.e. the
///   pre-Blossom first halving. Zebra's `halving(height, net) < 1` captures
///   the same invariant.
/// - Post-UPGRADE_YCASH, before [`ydf_mandate_end_height`]: 5% of the block
///   subsidy (`subsidy / 20`), ycashd's continuation of the founders' reward
///   in lieu of Zcash's funding-stream swap at Canopy.
/// - Otherwise: zero.
pub fn founders_reward(net: &Network, height: Height) -> Amount<NonNegative> {
    let Some(ycash_activation_height) = NetworkUpgrade::Ycash.activation_height(net) else {
        return Amount::zero();
    };

    let subsidy = block_subsidy(height, net)
        .expect("block subsidy must be valid for founders rewards");

    if height < ycash_activation_height {
        if halving(height, net) < 1 {
            subsidy.div_exact(5)
        } else {
            Amount::zero()
        }
    } else if height < net.ydf_mandate_end_height() {
        subsidy.div_exact(20)
    } else {
        Amount::zero()
    }
}
