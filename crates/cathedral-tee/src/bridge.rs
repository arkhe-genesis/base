use anyhow::Result;
use uuid::Uuid;

use crate::{
    secure_vm::SecureVmExecutor,
    types::{AttestationReport, AttestationResult, TeeType},
    verifier::Verifier,
};

pub struct TEEBridge {
    verifier: Verifier,
}

impl TEEBridge {
    pub fn new() -> Self {
        Self { verifier: Verifier::new() }
    }

    pub fn register_trusted_hash(&mut self, worker_id: &str, hash: &str) {
        self.verifier.register_trusted_hash(worker_id, hash);
    }

    pub fn verify(&self, report: &AttestationReport) -> AttestationResult {
        self.verifier.verify(report)
    }

    pub fn generate_challenge(&self) -> String {
        format!("challenge-{}", Uuid::new_v4())
    }

    pub fn get_tee_type(&self, _worker_id: &str) -> TeeType {
        TeeType::None
    }

    pub async fn verify_or_secure_execute(
        &self,
        worker_id: &str,
        binary: &[u16],
        input: &str,
    ) -> Result<String> {
        let tee_type = self.get_tee_type(worker_id);
        if tee_type == TeeType::None {
            // Usar software VM como TEE
            let vm_exec = SecureVmExecutor::new(100_000, 65536);
            vm_exec.execute_secure(binary, input).await
        } else {
            // Usar hardware TEE normal
            Ok("Executed in hardware TEE".to_string())
        }
    }
}
