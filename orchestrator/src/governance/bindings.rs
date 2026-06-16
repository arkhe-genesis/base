use ethers::prelude::abigen;

abigen!(
    CathedralConsensusLedger,
    "../governance/CathedralConsensusLedger.abi.json",
    event_derives(serde::Deserialize, serde::Serialize)
);
