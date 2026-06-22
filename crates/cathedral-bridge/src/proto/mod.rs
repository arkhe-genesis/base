pub struct ZkVerifyRequest {
    pub circuit_id: String,
    pub agent_id: String,
    pub proof_bytes: Vec<u8>,
    pub public_inputs: Vec<u8>,
    pub design_hash: String,
}

pub struct ZkVerifyResponse {
    pub valid: bool,
    pub circuit_id: String,
    pub verification_time_ms: String,
    pub error: Option<String>,
    pub verification_hash: Vec<u8>,
}

pub struct NostrPublishRequest {
    pub project_id: String,
    pub design_hash: String,
    pub tags: Vec<Vec<String>>,
    pub wormgraph_json: String,
    pub relay_urls: Vec<String>,
}

pub struct NostrPublishResponse {
    pub success: bool,
    pub event_id_hex: String,
    pub relay_urls: Vec<String>,
    pub error: Option<String>,
    pub published_at: u64,
}
