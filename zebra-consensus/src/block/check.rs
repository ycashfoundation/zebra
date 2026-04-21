//! Consensus check functions

use std::{collections::HashSet, sync::Arc};

use chrono::{DateTime, Utc};

use mset::MultiSet;
use zebra_chain::{
    amount::{
        Amount, DeferredPoolBalanceChange, Error as AmountError, NegativeAllowed, NonNegative,
    },
    block::{Block, Hash, Header, Height},
    parameters::{
        subsidy::{
            founders_reward, founders_reward_address, funding_stream_values, FundingStreamReceiver,
            SubsidyError,
        },
        Network, NetworkUpgrade,
    },
    transaction::{self, Transaction},
    transparent::{Address, Output},
    work::{
        difficulty::{ExpandedDifficulty, ParameterDifficulty as _},
        equihash,
    },
};

use crate::{error::*, funding_stream_address};

/// Checks if there is exactly one coinbase transaction in `Block`,
/// and if that coinbase transaction is the first transaction in the block.
/// Returns the coinbase transaction is successful.
///
/// > A transaction that has a single transparent input with a null prevout field,
/// > is called a coinbase transaction. Every block has a single coinbase
/// > transaction as the first transaction in the block.
///
/// <https://zips.z.cash/protocol/protocol.pdf#coinbasetransactions>
pub fn coinbase_is_first(block: &Block) -> Result<Arc<transaction::Transaction>, BlockError> {
    // # Consensus
    //
    // > A block MUST have at least one transaction
    //
    // <https://zips.z.cash/protocol/protocol.pdf#blockheader>
    let first = block
        .transactions
        .first()
        .ok_or(BlockError::NoTransactions)?;
    // > The first transaction in a block MUST be a coinbase transaction,
    // > and subsequent transactions MUST NOT be coinbase transactions.
    //
    // <https://zips.z.cash/protocol/protocol.pdf#blockheader>
    //
    // > A transaction that has a single transparent input with a null prevout
    // > field, is called a coinbase transaction.
    //
    // <https://zips.z.cash/protocol/protocol.pdf#coinbasetransactions>
    let mut rest = block.transactions.iter().skip(1);
    if !first.is_coinbase() {
        Err(TransactionError::CoinbasePosition)?;
    }
    // > A transparent input in a non-coinbase transaction MUST NOT have a null prevout
    //
    // <https://zips.z.cash/protocol/protocol.pdf#txnconsensus>
    if !rest.all(|tx| tx.is_valid_non_coinbase()) {
        Err(TransactionError::CoinbaseAfterFirst)?;
    }

    Ok(first.clone())
}

/// Returns `Ok(ExpandedDifficulty)` if the`difficulty_threshold` of `header` is at least as difficult as
/// the target difficulty limit for `network` (PoWLimit)
///
/// If the header difficulty threshold is invalid, returns an error containing `height` and `hash`.
pub fn difficulty_threshold_is_valid(
    header: &Header,
    network: &Network,
    height: &Height,
    hash: &Hash,
) -> Result<ExpandedDifficulty, BlockError> {
    let difficulty_threshold = header
        .difficulty_threshold
        .to_expanded()
        .ok_or(BlockError::InvalidDifficulty(*height, *hash))?;

    // Note: the comparison in this function is a u256 integer comparison, like
    // zcashd and bitcoin. Greater values represent *less* work.

    // The PowLimit check is part of `Threshold()` in the spec, but it doesn't
    // actually depend on any previous blocks.
    if difficulty_threshold > network.target_difficulty_limit() {
        Err(BlockError::TargetDifficultyLimit(
            *height,
            *hash,
            difficulty_threshold,
            network.clone(),
            network.target_difficulty_limit(),
        ))?;
    }

    Ok(difficulty_threshold)
}

/// Returns `Ok(())` if `hash` passes:
///   - the target difficulty limit for `network` (PoWLimit), and
///   - the difficulty filter,
///
/// based on the fields in `header`.
///
/// If the block is invalid, returns an error containing `height` and `hash`.
pub fn difficulty_is_valid(
    header: &Header,
    network: &Network,
    height: &Height,
    hash: &Hash,
) -> Result<(), BlockError> {
    let difficulty_threshold = difficulty_threshold_is_valid(header, network, height, hash)?;

    // Note: the comparison in this function is a u256 integer comparison, like
    // zcashd and bitcoin. Greater values represent *less* work.

    // # Consensus
    //
    // > The block MUST pass the difficulty filter.
    //
    // https://zips.z.cash/protocol/protocol.pdf#blockheader
    //
    // The difficulty filter is also context-free.
    if hash > &difficulty_threshold {
        Err(BlockError::DifficultyFilter(
            *height,
            *hash,
            difficulty_threshold,
            network.clone(),
        ))?;
    }

    Ok(())
}

/// Returns `Ok(())` if the `EquihashSolution` is valid for `header`
pub fn equihash_solution_is_valid(header: &Header) -> Result<(), equihash::Error> {
    // # Consensus
    //
    // > `solution` MUST represent a valid Equihash solution.
    //
    // https://zips.z.cash/protocol/protocol.pdf#blockheader
    header.solution.check(header)
}

/// Returns `Ok()` with the deferred pool balance change of the coinbase transaction if the block
/// subsidy in `block` is valid for `network`
///
/// [3.9]: https://zips.z.cash/protocol/protocol.pdf#subsidyconcepts
pub fn subsidy_is_valid(
    block: &Block,
    net: &Network,
    expected_block_subsidy: Amount<NonNegative>,
) -> Result<DeferredPoolBalanceChange, BlockError> {
    if expected_block_subsidy.is_zero() {
        return Ok(DeferredPoolBalanceChange::zero());
    }

    let height = block.coinbase_height().ok_or(SubsidyError::NoCoinbase)?;

    let mut coinbase_outputs: MultiSet<Output> = block
        .transactions
        .first()
        .ok_or(SubsidyError::NoCoinbase)?
        .outputs()
        .iter()
        .cloned()
        .collect();

    // Accepts both P2SH (pre-UPGRADE_YCASH legacy founder script) and P2PKH
    // (post-UPGRADE_YCASH Ycash founder address) outputs. `Address::script()`
    // emits the appropriate `scriptPubKey` for each.
    let mut has_amount = |addr: &Address, amount| {
        coinbase_outputs.remove(&Output::new(amount, addr.script()))
    };

    // Ycash founders' reward window: heights 1..YDF_MANDATE_END_HEIGHT. The
    // exact rule is split at UPGRADE_YCASH by `founders_reward` and
    // `founders_reward_address`:
    //
    // - Pre-UPGRADE_YCASH (heights 1..570_000 on Mainnet / 1..510_248 on
    //   Testnet): 20% of the block subsidy, paid to one of the 48 legacy
    //   Zcash founder P2SH addresses. Mirrors the Zcash pre-Canopy rule.
    // - Post-UPGRADE_YCASH, before `ydf_mandate_end_height`: 5% of the block
    //   subsidy, paid to one of the 48 Ycash founder addresses (typically
    //   P2PKH). Ycash's replacement for Zcash's Canopy funding-stream swap.
    //
    // After the YDF mandate end no founders' reward output is required.
    //
    // References:
    // - <https://zips.z.cash/protocol/protocol.pdf#foundersreward>
    // - `ycashd/src/main.cpp`, search for `UPGRADE_YCASH`
    if Height::MIN < height && height < net.ydf_mandate_end_height() {
        let addr = founders_reward_address(net, height).ok_or(BlockError::Other(format!(
            "founders reward address must be defined for height: {height:?}"
        )))?;

        if !has_amount(&addr, founders_reward(net, height)) {
            Err(SubsidyError::FoundersRewardNotFound)?;
        }
    }

    // Funding streams and NU6.1 lockbox disbursements. Ycash has no funding
    // streams and never activates NU6.1 on Mainnet or the default Testnet, so
    // both of the loops below iterate over empty sets by default. They remain
    // in place to handle custom testnets that configure either feature.
    //
    // References:
    // - <https://zips.z.cash/protocol/protocol.pdf#fundingstreams>
    // - [ZIP-271]: <https://zips.z.cash/zip-0271>
    // - [ZIP-1016]: <https://zips.z.cash/zip-101>

    let mut funding_streams = funding_stream_values(height, net, expected_block_subsidy)?;

    // The deferred pool contribution is checked in `miner_fees_are_valid()` according to
    // [ZIP-1015](https://zips.z.cash/zip-1015).
    let mut deferred_pool_balance_change = funding_streams
        .remove(&FundingStreamReceiver::Deferred)
        .unwrap_or_default()
        .constrain::<NegativeAllowed>()?;

    if Some(height) == NetworkUpgrade::Nu6_1.activation_height(net) {
        let lockbox_disbursements = net.lockbox_disbursements(height);

        if lockbox_disbursements.is_empty() {
            Err(BlockError::Other(
                "missing lockbox disbursements for NU6.1 activation block".to_string(),
            ))?;
        }

        deferred_pool_balance_change = lockbox_disbursements.into_iter().try_fold(
            deferred_pool_balance_change,
            |balance, (addr, expected_amount)| {
                if !has_amount(&addr, expected_amount) {
                    Err(SubsidyError::OneTimeLockboxDisbursementNotFound)?;
                }

                balance
                    .checked_sub(expected_amount)
                    .ok_or(SubsidyError::Underflow)
            },
        )?;
    };

    funding_streams.into_iter().try_for_each(
        |(receiver, expected_amount)| -> Result<(), BlockError> {
            let addr =
                funding_stream_address(height, net, receiver).ok_or(BlockError::Other(
                    "A funding stream other than the deferred pool must have an address"
                        .to_string(),
                ))?;

            if !has_amount(addr, expected_amount) {
                Err(SubsidyError::FundingStreamNotFound)?;
            }

            Ok(())
        },
    )?;

    Ok(DeferredPoolBalanceChange::new(deferred_pool_balance_change))
}

/// Returns `Ok(())` if the miner fees consensus rule is valid.
///
/// [7.1.2]: https://zips.z.cash/protocol/protocol.pdf#txnconsensus
pub fn miner_fees_are_valid(
    coinbase_tx: &Transaction,
    height: Height,
    block_miner_fees: Amount<NonNegative>,
    expected_block_subsidy: Amount<NonNegative>,
    expected_deferred_pool_balance_change: DeferredPoolBalanceChange,
    network: &Network,
) -> Result<(), BlockError> {
    let transparent_value_balance = coinbase_tx
        .outputs()
        .iter()
        .map(|output| output.value())
        .sum::<Result<Amount<NonNegative>, AmountError>>()
        .map_err(|_| SubsidyError::Overflow)?
        .constrain()
        .map_err(|e| BlockError::Other(format!("invalid transparent value balance: {e}")))?;
    let sapling_value_balance = coinbase_tx.sapling_value_balance().sapling_amount();
    let orchard_value_balance = coinbase_tx.orchard_value_balance().orchard_amount();

    // Coinbase transaction can still have a NSM deposit
    #[cfg(zcash_unstable = "zip235")]
    let zip233_amount: Amount<NegativeAllowed> = coinbase_tx
        .zip233_amount()
        .constrain()
        .map_err(|_| SubsidyError::InvalidZip233Amount)?;

    #[cfg(not(zcash_unstable = "zip235"))]
    let zip233_amount = Amount::zero();

    #[cfg(zcash_unstable = "zip235")]
    if let Some(nsm_activation_height) = NetworkUpgrade::Nu7.activation_height(network) {
        if height >= nsm_activation_height {
            let minimum_zip233_amount = ((block_miner_fees * 6).unwrap() / 10).unwrap();
            if zip233_amount < minimum_zip233_amount {
                Err(SubsidyError::InvalidZip233Amount)?
            }
        }
    }

    // # Consensus
    //
    // > - define the total output value of its coinbase transaction to be the total value in zatoshi of its transparent
    // >   outputs, minus vbalanceSapling, minus vbalanceOrchard, plus totalDeferredOutput(height);
    // > – define the total input value of its coinbase transaction to be the value in zatoshi of the block subsidy,
    // >   plus the transaction fees paid by transactions in the block.
    //
    // https://zips.z.cash/protocol/protocol.pdf#txnconsensus
    //
    // The expected lockbox funding stream output of the coinbase transaction is also subtracted
    // from the block subsidy value plus the transaction fees paid by transactions in this block.
    let total_output_value =
        (transparent_value_balance - sapling_value_balance - orchard_value_balance
            + expected_deferred_pool_balance_change.value()
            + zip233_amount)
            .map_err(|_| SubsidyError::Overflow)?;

    let total_input_value =
        (expected_block_subsidy + block_miner_fees).map_err(|_| SubsidyError::Overflow)?;

    // # Consensus
    //
    // > [Pre-NU6] The total output of a coinbase transaction MUST NOT be greater than its total
    // input.
    //
    // > [NU6 onward] The total output of a coinbase transaction MUST be equal to its total input.
    if if NetworkUpgrade::current(network, height) < NetworkUpgrade::Nu6 {
        total_output_value > total_input_value
    } else {
        total_output_value != total_input_value
    } {
        Err(SubsidyError::InvalidMinerFees)?
    };

    Ok(())
}

/// Returns `Ok(())` if `header.time` is less than or equal to
/// 2 hours in the future, according to the node's local clock (`now`).
///
/// This is a non-deterministic rule, as clocks vary over time, and
/// between different nodes.
///
/// "In addition, a full validator MUST NOT accept blocks with nTime
/// more than two hours in the future according to its clock. This
/// is not strictly a consensus rule because it is nondeterministic,
/// and clock time varies between nodes. Also note that a block that
/// is rejected by this rule at a given point in time may later be
/// accepted." [§7.5][7.5]
///
/// [7.5]: https://zips.z.cash/protocol/protocol.pdf#blockheader
///
/// If the header time is invalid, returns an error containing `height` and `hash`.
pub fn time_is_valid_at(
    header: &Header,
    now: DateTime<Utc>,
    height: &Height,
    hash: &Hash,
) -> Result<(), zebra_chain::block::BlockTimeError> {
    header.time_is_valid_at(now, height, hash)
}

/// Check Merkle root validity.
///
/// `transaction_hashes` is a precomputed list of transaction hashes.
///
/// # Consensus rules:
///
/// - A SHA-256d hash in internal byte order. The merkle root is derived from the
///   hashes of all transactions included in this block, ensuring that none of
///   those transactions can be modified without modifying the header. [7.6]
///
/// # Panics
///
/// - If block does not have a coinbase transaction.
///
/// [ZIP-244]: https://zips.z.cash/zip-0244
/// [7.1]: https://zips.z.cash/protocol/nu5.pdf#txnencodingandconsensus
/// [7.6]: https://zips.z.cash/protocol/nu5.pdf#blockheader
pub fn merkle_root_validity(
    network: &Network,
    block: &Block,
    transaction_hashes: &[transaction::Hash],
) -> Result<(), BlockError> {
    // TODO: deduplicate zebra-chain and zebra-consensus errors (#2908)
    block
        .check_transaction_network_upgrade_consistency(network)
        .map_err(|_| BlockError::WrongTransactionConsensusBranchId)?;

    let merkle_root = transaction_hashes.iter().cloned().collect();

    if block.header.merkle_root != merkle_root {
        return Err(BlockError::BadMerkleRoot {
            actual: merkle_root,
            expected: block.header.merkle_root,
        });
    }

    // Bitcoin's transaction Merkle trees are malleable, allowing blocks with
    // duplicate transactions to have the same Merkle root as blocks without
    // duplicate transactions.
    //
    // Collecting into a HashSet deduplicates, so this checks that there are no
    // duplicate transaction hashes, preventing Merkle root malleability.
    //
    // ## Full Block Validation
    //
    // Duplicate transactions should cause a block to be
    // rejected, as duplicate transactions imply that the block contains a
    // double-spend. As a defense-in-depth, however, we also check that there
    // are no duplicate transaction hashes.
    //
    // ## Checkpoint Validation
    //
    // To prevent malleability (CVE-2012-2459), we also need to check
    // whether the transaction hashes are unique.
    if transaction_hashes.len() != transaction_hashes.iter().collect::<HashSet<_>>().len() {
        return Err(BlockError::DuplicateTransaction);
    }

    Ok(())
}
