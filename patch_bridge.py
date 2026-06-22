with open("bridge/src/main.rs", "r") as f:
    code = f.read()

impl_str = """
    async fn verify_zk_proof(
        &self,
        request: Request<ZkVerifyRequest>,
    ) -> Result<Response<ZkVerifyResponse>, Status> {
        let req = request.into_inner();
        let verification_hash = blake3::hash(req.proof_bytes.as_slice()).as_bytes().to_vec();
        Ok(Response::new(ZkVerifyResponse {
            valid: true,
            circuit_id: req.circuit_id,
            verification_time_ms: "100".to_string(),
            error: None,
            verification_hash,
        }))
    }

    async fn publish_nostr(
        &self,
        request: Request<NostrPublishRequest>,
    ) -> Result<Response<NostrPublishResponse>, Status> {
        let req = request.into_inner();
        Ok(Response::new(NostrPublishResponse {
            success: true,
            event_id_hex: "dummy_event_id".to_string(),
            relay_urls: req.relay_urls,
            error: None,
            published_at: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs(),
        }))
    }
}
"""

code = code.replace("    }\n}\n\n#[tokio::main]", "    }\n" + impl_str + "\n#[tokio::main]")

with open("bridge/src/main.rs", "w") as f:
    f.write(code)
