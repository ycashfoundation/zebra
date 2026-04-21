//! Testnet-specific constants for block subsidies.
//!
//! Ycash continues paying a founders' reward past Canopy (instead of Zcash's
//! funding-stream swap) and never activates NU6, NU6.1, or any later upgrade,
//! so the funding-stream table is empty on testnet. See
//! [`crate::parameters::network::subsidy::founders_reward`] for the two-phase
//! founders' reward rule that replaces funding streams on Ycash.

use lazy_static::lazy_static;

use crate::parameters::subsidy::FundingStreams;

/// The height at which Ycash's YDF mandate ends on Testnet, per
/// `nYdfMandateEndHeight` in `ycashd/src/chainparams.cpp` (v4.4.4-1).
pub(crate) const YDF_MANDATE_END_HEIGHT: u32 = 900_000;

/// Post-UPGRADE_YCASH address-change interval, shared with Mainnet.
pub(crate) const YCASH_FOUNDER_ADDRESS_CHANGE_INTERVAL: u32 = 17_917;

/// Number of founder addresses on Testnet (pre-UPGRADE_YCASH).
pub(crate) const NUM_FOUNDER_ADDRESSES: usize = 48;

/// Number of Ycash founder addresses on Testnet (post-UPGRADE_YCASH).
pub(crate) const NUM_YCASH_FOUNDER_ADDRESSES: usize = 48;

/// List of pre-UPGRADE_YCASH founder addresses on Testnet, per
/// `vFoundersRewardAddress` in `ycashd/src/chainparams.cpp`. See
/// [`super::mainnet::FOUNDER_ADDRESS_LIST`] for the rationale for keeping the
/// legacy Zcash encoding.
pub(crate) const FOUNDER_ADDRESS_LIST: [&str; NUM_FOUNDER_ADDRESSES] = [
    "t2UNzUUx8mWBCRYPRezvA363EYXyEpHokyi",
    "t2N9PH9Wk9xjqYg9iin1Ua3aekJqfAtE543",
    "t2NGQjYMQhFndDHguvUw4wZdNdsssA6K7x2",
    "t2ENg7hHVqqs9JwU5cgjvSbxnT2a9USNfhy",
    "t2BkYdVCHzvTJJUTx4yZB8qeegD8QsPx8bo",
    "t2J8q1xH1EuigJ52MfExyyjYtN3VgvshKDf",
    "t2Crq9mydTm37kZokC68HzT6yez3t2FBnFj",
    "t2EaMPUiQ1kthqcP5UEkF42CAFKJqXCkXC9",
    "t2F9dtQc63JDDyrhnfpzvVYTJcr57MkqA12",
    "t2LPirmnfYSZc481GgZBa6xUGcoovfytBnC",
    "t26xfxoSw2UV9Pe5o3C8V4YybQD4SESfxtp",
    "t2D3k4fNdErd66YxtvXEdft9xuLoKD7CcVo",
    "t2DWYBkxKNivdmsMiivNJzutaQGqmoRjRnL",
    "t2C3kFF9iQRxfc4B9zgbWo4dQLLqzqjpuGQ",
    "t2MnT5tzu9HSKcppRyUNwoTp8MUueuSGNaB",
    "t2AREsWdoW1F8EQYsScsjkgqobmgrkKeUkK",
    "t2Vf4wKcJ3ZFtLj4jezUUKkwYR92BLHn5UT",
    "t2K3fdViH6R5tRuXLphKyoYXyZhyWGghDNY",
    "t2VEn3KiKyHSGyzd3nDw6ESWtaCQHwuv9WC",
    "t2F8XouqdNMq6zzEvxQXHV1TjwZRHwRg8gC",
    "t2BS7Mrbaef3fA4xrmkvDisFVXVrRBnZ6Qj",
    "t2FuSwoLCdBVPwdZuYoHrEzxAb9qy4qjbnL",
    "t2SX3U8NtrT6gz5Db1AtQCSGjrpptr8JC6h",
    "t2V51gZNSoJ5kRL74bf9YTtbZuv8Fcqx2FH",
    "t2FyTsLjjdm4jeVwir4xzj7FAkUidbr1b4R",
    "t2EYbGLekmpqHyn8UBF6kqpahrYm7D6N1Le",
    "t2NQTrStZHtJECNFT3dUBLYA9AErxPCmkka",
    "t2GSWZZJzoesYxfPTWXkFn5UaxjiYxGBU2a",
    "t2RpffkzyLRevGM3w9aWdqMX6bd8uuAK3vn",
    "t2JzjoQqnuXtTGSN7k7yk5keURBGvYofh1d",
    "t2AEefc72ieTnsXKmgK2bZNckiwvZe3oPNL",
    "t2NNs3ZGZFsNj2wvmVd8BSwSfvETgiLrD8J",
    "t2ECCQPVcxUCSSQopdNquguEPE14HsVfcUn",
    "t2JabDUkG8TaqVKYfqDJ3rqkVdHKp6hwXvG",
    "t2FGzW5Zdc8Cy98ZKmRygsVGi6oKcmYir9n",
    "t2DUD8a21FtEFn42oVLp5NGbogY13uyjy9t",
    "t2UjVSd3zheHPgAkuX8WQW2CiC9xHQ8EvWp",
    "t2TBUAhELyHUn8i6SXYsXz5Lmy7kDzA1uT5",
    "t2Tz3uCyhP6eizUWDc3bGH7XUC9GQsEyQNc",
    "t2NysJSZtLwMLWEJ6MH3BsxRh6h27mNcsSy",
    "t2KXJVVyyrjVxxSeazbY9ksGyft4qsXUNm9",
    "t2J9YYtH31cveiLZzjaE4AcuwVho6qjTNzp",
    "t2QgvW4sP9zaGpPMH1GRzy7cpydmuRfB4AZ",
    "t2NDTJP9MosKpyFPHJmfjc5pGCvAU58XGa4",
    "t29pHDBWq7qN4EjwSEHg8wEqYe9pkmVrtRP",
    "t2Ez9KM8VJLuArcxuEkNRAkhNvidKkzXcjJ",
    "t2D5y7J5fpXajLbGrMBQkFg2mFN8fo3n8cX",
    "t2UV2wr1PTaUiybpkV3FdSdGxUJeZdZztyt",
];

/// List of post-UPGRADE_YCASH Ycash founder addresses on Testnet, per
/// `vYcashFoundersRewardAddress` in `ycashd/src/chainparams.cpp`.
pub(crate) const YCASH_FOUNDER_ADDRESS_LIST: [&str; NUM_YCASH_FOUNDER_ADDRESSES] = [
    "smDw2LWkeuJ1NGBDDZvdNbzY8A9D1mkkDZm",
    "smDxM6WPpz3HcK6m9cCnhQkBXMLnUf3cryA",
    "smEKdQPcZHYTmcTbVkqfWryRbEZWMrapjMo",
    "smEVfJmuGErW6ZM3XSNwbJR6cPU3iPABAqY",
    "smEkWMsbV1CBZosu9wtq69f2vNXBT1owNKe",
    "smFd3Dh5MjEttRHd9S8kx153Vzesefzjc2d",
    "smGBXB9SrjnEDf7ASQxvnujBRc1qBb58o5q",
    "smGLTYjSriA3n8EMf4JTiHLGUCzUYjau3WV",
    "smGVF2kDywxhjfzBqoFDE1AyXZEafTLSjbH",
    "smGVrxUHUzd2gaURPw2ASoE3L5WMFcxJcp1",
    "smHTCd59Q9pFzrzA73f6het1ozDzeQaA8E3",
    "smHy6JaGM9gkaGBJ4DF4p5FbFvoUbpAusWd",
    "smJ1fpQdKNuchkxuVUMBkcCWoBcWFmyDAyZ",
    "smJ2j3Gea5XH7ERpyYzvo6YQKaoYfzkMUS3",
    "smJ8gtE5EX5oFSp4c5cCDpxoajXTQu73VSC",
    "smJyGxvwpCxPaFJM6TwZ6cwT1qz5PMPPBeL",
    "smKEt1iVgCDY9V4915HnGpFK3zyTPyQixMa",
    "smKXxXsNLUVE8Qro6R9EyaEZmvgxqjbsFX9",
    "smLTH7FEiXUVpWjhoL91ToMoSZPU8xAvEh9",
    "smLo65rNyiEYiVHxWZHNNP9QU1HsQu3QX7b",
    "smMDUN36MqFE4thY54CQnWBpU4ePuayF9TV",
    "smMWmoipQ7YVuRAJaQHKqhHbbTg28mS6ET5",
    "smMtCCZsR3s7ZiEYskFietNPfSb1eVHNoZi",
    "smMuh9QVkpQg93Gb54ucaVykbk9FgjZBN3D",
    "smNM44GrsbMWHQRhhayqieCadzURNDLkxkW",
    "smNhJLQs3LmHWtVscrnJmrLhhjPcDD3unJZ",
    "smPHxC1438rANivn1omntbRF5Nf2wSZfFhs",
    "smSAidYKoFY2fmi2efcoJPSeBpdVC2vyHvj",
    "smShCYXefsx8RcnW2b7duPKBcD5TXWY5K2A",
    "smTRxVMtveLrdzVb1B9rLKfDZ3Qp8kAna7r",
    "smTnar3ernxTG5voae2bUC185EicBdSKVux",
    "smUmPeP8VwuPsNTcJcB7YZY1b1HJJsJfkYp",
    "smVSGTzPP4dbj3HXRR84WoBFWw4EVuuyVi2",
    "smW8AA9LAKGMj4EXxNtfJQFLFaiuzTyGwYa",
    "smWAgHkGiQ1fZHF9ZBjYzKS7ZX8JtjvWbaU",
    "smX1HT3n9mtPGeK7qMBETCEGL5UG8eC8nmy",
    "smX3udkLyb3qZ2z2muEinbvNuSi5M2JiiE5",
    "smXEcB2QZaZgevkB7CS1Tz2BZNQ9cnNwXBj",
    "smXddMXWZvpRDq4FjuTGxLWYTtwbNwc36g5",
    "smXfPS6G7aiVECG8qFvZc9oFe1bzEvCKECG",
    "smXxQi63m9x2WfhdBYogWdKzavVpbnnGyJq",
    "smZ53EtRafyjGZyjiNDb1FiGbwXbtE3aqTB",
    "smZJPM9KsKAdfotD6Woh2nb8wLedvhf4Nw5",
    "smZyciv3CGZLAFU8d2yKff33FU8f2nek5yx",
    "smZzRpCLENnxN5JgMHaKtMruTCi7jsa7Tak",
    "smaWKSqvUJvajRquGbaHBdNbL78fDf6hdwE",
    "smazMQ9G7NLJXAzX4ZMKc8x6DigyeoEucgk",
    "smbTaZmsKNEVstoQVyZcJAMJExiytfnwaMU",
];

lazy_static! {
    /// Funding streams for Testnet. Empty on Ycash — see Mainnet docs.
    pub(crate) static ref FUNDING_STREAMS: Vec<FundingStreams> = Vec::new();
}
