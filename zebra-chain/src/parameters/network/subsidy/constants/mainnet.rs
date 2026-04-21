//! Mainnet-specific constants for block subsidies.
//!
//! Ycash continues paying a founders' reward past Canopy (instead of Zcash's
//! funding-stream swap) and never activates NU6, NU6.1, or any later upgrade,
//! so the funding-stream table is empty on mainnet. See
//! [`crate::parameters::network::subsidy::founders_reward`] for the two-phase
//! founders' reward rule that replaces funding streams on Ycash.

use lazy_static::lazy_static;

use crate::parameters::subsidy::FundingStreams;

/// The height at which Ycash's YDF mandate ends on Mainnet, per
/// `nYdfMandateEndHeight` in `ycashd/src/chainparams.cpp`. Blocks at heights
/// `1..YDF_MANDATE_END_HEIGHT` (excluding the end) pay a founders' reward;
/// at or after this height no founders' reward is required.
pub(crate) const YDF_MANDATE_END_HEIGHT: u32 = 2_275_000;

/// Post-UPGRADE_YCASH address-change interval (roughly one month at 150s
/// target spacing), per `ycashd/src/chainparams.cpp`
/// `GetFoundersRewardAddressAtHeight`.
pub(crate) const YCASH_FOUNDER_ADDRESS_CHANGE_INTERVAL: u32 = 17_917;

/// Number of founder addresses on Mainnet (pre-UPGRADE_YCASH).
pub(crate) const NUM_FOUNDER_ADDRESSES: usize = 48;

/// Number of Ycash founder addresses on Mainnet (post-UPGRADE_YCASH).
pub(crate) const NUM_YCASH_FOUNDER_ADDRESSES: usize = 48;

/// List of pre-UPGRADE_YCASH founder addresses on Mainnet, per
/// `vFoundersRewardAddress` in `ycashd/src/chainparams.cpp`. These are the
/// original 48 Zcash founder addresses; ycashd re-encodes them with Ycash
/// base58 prefixes at validation time (`keyIO.ZecToYec`). In Zebra we store
/// them in their original Zcash form — `Address::FromStr` accepts the legacy
/// Zcash prefixes, and the parsed `transparent::Address` is stringified with
/// Ycash prefixes by `NetworkKind::b58_*_address_prefix()` on output.
pub(crate) const FOUNDER_ADDRESS_LIST: [&str; NUM_FOUNDER_ADDRESSES] = [
    "t3Vz22vK5z2LcKEdg16Yv4FFneEL1zg9ojd",
    "t3cL9AucCajm3HXDhb5jBnJK2vapVoXsop3",
    "t3fqvkzrrNaMcamkQMwAyHRjfDdM2xQvDTR",
    "t3TgZ9ZT2CTSK44AnUPi6qeNaHa2eC7pUyF",
    "t3SpkcPQPfuRYHsP5vz3Pv86PgKo5m9KVmx",
    "t3Xt4oQMRPagwbpQqkgAViQgtST4VoSWR6S",
    "t3ayBkZ4w6kKXynwoHZFUSSgXRKtogTXNgb",
    "t3adJBQuaa21u7NxbR8YMzp3km3TbSZ4MGB",
    "t3K4aLYagSSBySdrfAGGeUd5H9z5Qvz88t2",
    "t3RYnsc5nhEvKiva3ZPhfRSk7eyh1CrA6Rk",
    "t3Ut4KUq2ZSMTPNE67pBU5LqYCi2q36KpXQ",
    "t3ZnCNAvgu6CSyHm1vWtrx3aiN98dSAGpnD",
    "t3fB9cB3eSYim64BS9xfwAHQUKLgQQroBDG",
    "t3cwZfKNNj2vXMAHBQeewm6pXhKFdhk18kD",
    "t3YcoujXfspWy7rbNUsGKxFEWZqNstGpeG4",
    "t3bLvCLigc6rbNrUTS5NwkgyVrZcZumTRa4",
    "t3VvHWa7r3oy67YtU4LZKGCWa2J6eGHvShi",
    "t3eF9X6X2dSo7MCvTjfZEzwWrVzquxRLNeY",
    "t3esCNwwmcyc8i9qQfyTbYhTqmYXZ9AwK3X",
    "t3M4jN7hYE2e27yLsuQPPjuVek81WV3VbBj",
    "t3gGWxdC67CYNoBbPjNvrrWLAWxPqZLxrVY",
    "t3LTWeoxeWPbmdkUD3NWBquk4WkazhFBmvU",
    "t3P5KKX97gXYFSaSjJPiruQEX84yF5z3Tjq",
    "t3f3T3nCWsEpzmD35VK62JgQfFig74dV8C9",
    "t3Rqonuzz7afkF7156ZA4vi4iimRSEn41hj",
    "t3fJZ5jYsyxDtvNrWBeoMbvJaQCj4JJgbgX",
    "t3Pnbg7XjP7FGPBUuz75H65aczphHgkpoJW",
    "t3WeKQDxCijL5X7rwFem1MTL9ZwVJkUFhpF",
    "t3Y9FNi26J7UtAUC4moaETLbMo8KS1Be6ME",
    "t3aNRLLsL2y8xcjPheZZwFy3Pcv7CsTwBec",
    "t3gQDEavk5VzAAHK8TrQu2BWDLxEiF1unBm",
    "t3Rbykhx1TUFrgXrmBYrAJe2STxRKFL7G9r",
    "t3aaW4aTdP7a8d1VTE1Bod2yhbeggHgMajR",
    "t3YEiAa6uEjXwFL2v5ztU1fn3yKgzMQqNyo",
    "t3g1yUUwt2PbmDvMDevTCPWUcbDatL2iQGP",
    "t3dPWnep6YqGPuY1CecgbeZrY9iUwH8Yd4z",
    "t3QRZXHDPh2hwU46iQs2776kRuuWfwFp4dV",
    "t3enhACRxi1ZD7e8ePomVGKn7wp7N9fFJ3r",
    "t3PkLgT71TnF112nSwBToXsD77yNbx2gJJY",
    "t3LQtHUDoe7ZhhvddRv4vnaoNAhCr2f4oFN",
    "t3fNcdBUbycvbCtsD2n9q3LuxG7jVPvFB8L",
    "t3dKojUU2EMjs28nHV84TvkVEUDu1M1FaEx",
    "t3aKH6NiWN1ofGd8c19rZiqgYpkJ3n679ME",
    "t3MEXDF9Wsi63KwpPuQdD6by32Mw2bNTbEa",
    "t3WDhPfik343yNmPTqtkZAoQZeqA83K7Y3f",
    "t3PSn5TbMMAEw7Eu36DYctFezRzpX1hzf3M",
    "t3R3Y5vnBLrEn8L6wFjPjBLnxSUQsKnmFpv",
    "t3Pcm737EsVkGTbhsu2NekKtJeG92mvYyoN",
];

/// List of post-UPGRADE_YCASH Ycash founder addresses on Mainnet, per
/// `vYcashFoundersRewardAddress` in `ycashd/src/chainparams.cpp`. These are
/// selected by `(height - UPGRADE_YCASH_height) / YCASH_FOUNDER_ADDRESS_CHANGE_INTERVAL`,
/// modulo the list length — the list cycles continuously until
/// `YDF_MANDATE_END_HEIGHT`.
pub(crate) const YCASH_FOUNDER_ADDRESS_LIST: [&str; NUM_YCASH_FOUNDER_ADDRESSES] = [
    "s1hfWJ4ej1H3s8XCUb7YnrU68K64AsGVUHE",
    "s1iZaRoYtafWspcieQxg6hhaU4DfZyAdGQf",
    "s1RSr6xec6Cc98emM4cdq45rkVekHMjRWbw",
    "s1RsqYeweoKVepivLPLsiajE8c6khu5UKhS",
    "s1MNmqMWyV4nMWE4oDb1nqJs7haJrv9QTKp",
    "s1RP95ESdcu33gMtU7deLW9TP6yDZncjRQ7",
    "s1h8W7xQbiU8Zxu21Zcg82NByjkWMcEbNtX",
    "s1PVcdfcrJrDCmXxgSTGuKGSNhwSYZ51XKJ",
    "s1PkV5nFkgQN4EGuTtEcmm4CxeBVx2L5HHv",
    "s1jiVSTfMaFUrWnf17BGc416oomHbut58Ue",
    "s1Zr2KdHtnK2zNSMQDrAVv3KU51mgDbqgwe",
    "s1QkY6tmBHPZacXPMPmsjP37Kxgs5mcgcAn",
    "s1Xu76ZmGDENdLFAiuj5iMdp1RA4hWSNieq",
    "s1bdiEnfBYaEgrt2TmnY3ZHmdhg5AEw9tjN",
    "s1asM9Ui4U13GjmLoAhvfK6J5QihemQR9Pk",
    "s1QhTSXYu4K1cTNomN27wiep9WC9HBZjrxJ",
    "s1j3Ef2qCNjwRAM18BgwsPAZFzZ475BWM5S",
    "s1QZibiN7iqVCfVBES9Gn7e3o5psxRKtpwE",
    "s1fdiDZHkzp8K8UajpVwYUdyFeb6jNVyoKv",
    "s1iMShbVRH1eCGxK2ZoLMDn5o9NcwXkNPVF",
    "s1YtUXAMt8m31gGeP5m3Y53B1wrMk3FFigJ",
    "s1gy9aqWUihGRjZa3vqc7136vqTGNAWyefF",
    "s1NNozrex18HZqcHCGpGoSRkj8hqHLEPaVC",
    "s1NYNDdqthMf7D7sZbnLGuecDtXb48Ne2bf",
    "s1P7UJ9Wp7jstJPUbvMSRVFjN8tfQueQSK3",
    "s1RDeyH7xg8y9veb9XfAmtKzrMTjFS14c4T",
    "s1NmH6MNXU19xoHjUfpQpb4dEMSy1Wbs9tC",
    "s1UnFL2yrZapMKmB5EqaBKpogZmnXLUgELB",
    "s1XT3W1sLFdgmGoecQbPdJbjUDMurW8CFA2",
    "s1gyVwgangQxLCAcm8VS4SWqXDqeohNg7hd",
    "s1k3eWbqnbVM1xtZEDc81UbFdwXgXNnbtdH",
    "s1Wr6eAh3gZWwBVRZcND9YCzdNfR9cARkD4",
    "s1dtjp2KHWZ6qF2LvgNiEwjJV2dA2c6y75V",
    "s1cjQf9kmjQdTmnn6mbBaesHMNLt2JqjyEV",
    "s1URQeusSoi7fkgyAwCshFzobUzmLGH4U3b",
    "s1Z9YqM2h48HUf8kcSHS89q4Z6Bg9xua3kA",
    "s1TLmZzMDsDhYfh4vpY7NpRB4kao2UEEqKu",
    "s1QEWvfC1uifDfi78NY7cArw9xLEja7QAZR",
    "s1b4kfW9WMUtd2H7X4C64KLzqPWdMPXRMtS",
    "s1cHTXzCXhKYAX7sY7D8YGcmopjN8Yngoju",
    "s1QKjMQDeF9FLVo2sL8m11VC4ZA18s61s2K",
    "s1gV8D561ZpmaZVxG176cQM1bMFMnHLvujE",
    "s1caQmLCYVDZegcMoBckHD2RXjBh7ikpj2j",
    "s1Y63AsWsJTk5t5nSZfaFcFWmtfnFUUAu2V",
    "s1Y2U4GsfZdP9LAbC97GAmSdihBX5FU9gQn",
    "s1bPYWZMXzyN2ML2vswDiCckmas775QFs2Q",
    "s1erG25RcWYCiBPbT7khTU4ULhzm8jJZ7pv",
    "s1kYEiPdFZ3oV389q2MmSYY932qPF1ygVtx",
];

lazy_static! {
    /// Funding streams for Mainnet. Empty on Ycash: post-Canopy subsidy is
    /// delivered via the Ycash founders' reward through
    /// [`YDF_MANDATE_END_HEIGHT`] rather than ECC/ZF/MG/FPF streams.
    pub(crate) static ref FUNDING_STREAMS: Vec<FundingStreams> = Vec::new();
}
