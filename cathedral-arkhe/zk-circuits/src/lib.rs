pub mod physical_constraint;
pub mod reputation_circuit;

pub use physical_constraint::{PhysicalConstraintProofGenerator, PhysicalConstraintType};

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ZkProof {
    pub proof_bytes: Vec<u8>,
    pub public_inputs: Vec<u64>,
    pub circuit_id: String,
    pub verification_key_hash: String,
}

pub trait ZkBackend: Send + Sync {
    fn generate_proof(&self, circuit_id: &str, public_inputs: &[u8], private_inputs: &[u8]) -> anyhow::Result<ZkProof>;
    fn verify_proof(&self, proof: &ZkProof) -> anyhow::Result<bool>;
    fn clone_box(&self) -> Box<dyn ZkBackend + Send + Sync> where Self: Clone + 'static { Box::new(self.clone()) }
}
