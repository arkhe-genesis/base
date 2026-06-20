use tonic::{transport::Server, Request, Response, Status};
use serde_json::Value;
use std::collections::HashMap;
use ed25519_dalek::{Signature, VerifyingKey, Verifier};
use std::convert::TryInto;
use zstd::stream::read::Decoder;
use std::io::Read;

pub mod cathedral {
    pub mod v1 {
        tonic::include_proto!("cathedral.v1");
    }
}

use cathedral::v1::cathedral_bridge_server::{CathedralBridge, CathedralBridgeServer};
use cathedral::v1::{
    IngestRequest, IngestResponse, GovernanceRequest, GovernanceResponse,
    QueryProvenanceRequest, QueryProvenanceResponse, GovernanceVerdict,
};

const MAX_DECOMPRESSED_SIZE: u64 = 5 * 1024 * 1024; // 5 MB limit

#[derive(Default)]
pub struct MyCathedralBridge {
    pub agent_keys: HashMap<String, VerifyingKey>,
}

impl MyCathedralBridge {
    pub fn decompress_payload(base64_data: &str) -> Result<Vec<u8>, Status> {
        use base64::{Engine as _, engine::general_purpose::STANDARD};

        let compressed = STANDARD.decode(base64_data).map_err(|e| Status::invalid_argument(format!("Invalid base64 payload: {}", e)))?;

        let decoder = Decoder::new(compressed.as_slice()).map_err(|e| Status::invalid_argument(format!("Failed to init zstd decoder: {}", e)))?;
        let mut limited_decoder = decoder.take(MAX_DECOMPRESSED_SIZE);

        let mut decompressed = Vec::new();
        limited_decoder.read_to_end(&mut decompressed).map_err(|e| Status::invalid_argument(format!("Failed to decompress: {}", e)))?;
        Ok(decompressed)
    }

    pub fn verify_signature(&self, agent_id: &str, message: &[u8], signature_hex: &str) -> Result<(), Status> {
        let key = self.agent_keys.get(agent_id).ok_or_else(|| Status::unauthenticated("Unknown agent_id"))?;

        let sig_bytes = hex::decode(signature_hex).map_err(|_| Status::invalid_argument("Invalid hex signature"))?;
        if sig_bytes.len() != 64 {
            return Err(Status::invalid_argument("Invalid signature length"));
        }

        let signature = Signature::from_bytes(sig_bytes.as_slice().try_into().unwrap());

        key.verify(message, &signature).map_err(|_| Status::unauthenticated("Invalid signature"))?;

        Ok(())
    }
}

#[tonic::async_trait]
impl CathedralBridge for MyCathedralBridge {
    async fn ingest(
        &self,
        request: Request<IngestRequest>,
    ) -> Result<Response<IngestResponse>, Status> {
        let compressed = request.metadata().get("x-payload-compressed").map(|v| v.to_str().unwrap_or("false") == "true").unwrap_or(false);
        let signature_opt = request.metadata().get("x-agent-signature").map(|v| v.to_str().unwrap_or("").to_string());

        let req = request.into_inner();
        println!("Received ingest for project: {}", req.project_id);

        if let Some(_signature) = signature_opt {
            let _message = format!("{}:{}", req.project_id, req.agent_id);
            // self.verify_signature(&req.agent_id, _message.as_bytes(), &_signature)?;
        }

        let mut events_accepted = 0;
        let mut rejected_event_ids = vec![];

        for mut event in req.events {
            if compressed {
                if let Ok(decomp_bytes) = Self::decompress_payload(&event.payload_json) {
                    event.payload_json = String::from_utf8(decomp_bytes).unwrap_or_default();
                } else {
                    rejected_event_ids.push(event.event_id);
                    continue;
                }
            }
            events_accepted += 1;
        }

        let reply = IngestResponse {
            success: rejected_event_ids.is_empty(),
            message: "Events processed".to_string(),
            events_accepted,
            rejected_event_ids,
        };
        Ok(Response::new(reply))
    }

    async fn request_governance(
        &self,
        request: Request<GovernanceRequest>,
    ) -> Result<Response<GovernanceResponse>, Status> {
        let req = request.into_inner();
        let reply = GovernanceResponse {
            request_id: req.request_id,
            verdict: GovernanceVerdict::Approved as i32,
            rationale: "Auto-approved by default policy".to_string(),
            conditions: vec![],
            evaluated_by: "grpc_server".to_string(),
            evaluated_at: Some(prost_types::Timestamp {
                seconds: std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_secs() as i64,
                nanos: 0,
            }),
        };
        Ok(Response::new(reply))
    }

    async fn query_provenance(
        &self,
        _request: Request<QueryProvenanceRequest>,
    ) -> Result<Response<QueryProvenanceResponse>, Status> {
        Err(Status::unimplemented("Not yet implemented"))
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let addr = "[::1]:50051".parse()?;

    let mut bridge = MyCathedralBridge::default();

    let public_key_bytes = [0u8; 32];
    if let Ok(vk) = VerifyingKey::from_bytes(&public_key_bytes) {
        bridge.agent_keys.insert("default-agent".to_string(), vk);
    }

    println!("CathedralBridge server listening on {}", addr);

    Server::builder()
        .add_service(CathedralBridgeServer::new(bridge))
        .serve(addr)
        .await?;

    Ok(())
}
