use std::sync::Arc;
use tonic::{Request, Response, Status};
use tracing::info;

use crate::proto::{ZkVerifyRequest, ZkVerifyResponse};
use crate::server::BridgeState;
use cathedral_zk_risc0::Risc0Verifier;
use cathedral_zk_circuits::{ZkProof as CircuitProof, ZkPublicInputs};

pub struct ZkHandler;

impl ZkHandler {
    pub async fn handle(
        state: Arc<BridgeState>,
        request: Request<ZkVerifyRequest>,
    ) -> Result<Response<ZkVerifyResponse>, Status> {
        let req = request.into_inner();
        info!("🔐 VerifyZkProof: circuit={}, agent={}", req.circuit_id, req.agent_id);

        // 1. Busca a chave de verificação (em produção, de um registro)
        let vk = state.verification_keys
            .read()
            .await
            .get(&req.circuit_id)
            .cloned()
            .ok_or_else(|| Status::not_found(format!("Circuito '{}' não encontrado", req.circuit_id)))?;

        // 2. Constrói a prova
        let proof = CircuitProof {
            proof_bytes: req.proof_bytes.clone(),
            public_inputs: req.public_inputs.clone(),
            circuit_id: req.circuit_id.clone(),
            verification_key_hash: vk.hash.clone(),
        };

        // 3. Usa o backend RISC Zero para verificar
        let verifier = Risc0Verifier::new(&vk.elf).map_err(|e| Status::internal(e))?;
        let public_inputs = ZkPublicInputs {
            inputs: proof.public_inputs.clone(),
        };

        let valid = verifier.verify(&proof, &public_inputs)
            .map_err(|e| Status::internal(format!("Erro na verificação: {}", e)))?;

        // 4. Registra no WormGraph (proveniência)
        if valid {
            let entry = cathedral_wormgraph::ProvenanceEntry {
                id: uuid::Uuid::new_v4().to_string(),
                version: 0,
                decision_type: "ZkVerification".to_string(),
                before_state: "{}".to_string(),
                after_state: serde_json::json!({
                    "circuit": req.circuit_id,
                    "agent": req.agent_id,
                    "design_hash": req.design_hash,
                    "valid": valid,
                }).to_string(),
                rationale: Some("Prova ZK verificada com sucesso".to_string()),
                timestamp: chrono::Utc::now().timestamp(),
                agent_id: req.agent_id.clone(),
                entry_hash: blake3::hash(req.proof_bytes.as_slice()).as_bytes().to_vec(),
                nostr_event_id: None,
                tree_id: None,
                parent_event_id: None,
                agent_identity: None,
            };
            let _ = state.wormgraph.append(entry).await;
        }

        let verification_hash = blake3::hash(req.proof_bytes.as_slice()).as_bytes().to_vec();

        info!("✅ Prova ZK verificada: valid={}", valid);
        Ok(Response::new(ZkVerifyResponse {
            valid,
            circuit_id: req.circuit_id,
            verification_time_ms: "0".to_string(), // Em produção, medir tempo
            error: if valid { None } else { Some("Prova inválida".to_string()) },
            verification_hash,
        }))
    }
}
