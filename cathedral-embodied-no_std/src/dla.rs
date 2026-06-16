pub struct MemoryProof {
    pub merkle_root: String,
}
pub async fn prove_memory_state() -> Result<MemoryProof, String> {
    Ok(MemoryProof { merkle_root: "".to_string() })
}
