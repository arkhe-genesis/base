pub struct ZkProof {
    pub proof_bytes: Vec<u8>,
    pub public_inputs: Vec<u8>,
    pub circuit_id: String,
    pub verification_key_hash: Vec<u8>,
}
pub struct ZkPublicInputs {
    pub inputs: Vec<u8>,
}
pub struct ZkBackend;
