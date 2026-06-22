use std::collections::HashMap;

use tokio::sync::RwLock;

pub struct BridgeState {
    pub verification_keys: RwLock<HashMap<String, VerificationKey>>,
    pub wormgraph: cathedral_wormgraph::WormGraph,
    pub nostr_replicator: Option<cathedral_nostr::NostrReplicator>,
}

#[derive(Clone)]
pub struct VerificationKey {
    pub elf: Vec<u8>,
    pub hash: Vec<u8>,
}
