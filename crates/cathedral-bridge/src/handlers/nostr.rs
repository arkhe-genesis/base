use std::sync::Arc;

use tonic::{Request, Response, Status};
use tracing::info;

use crate::{
    proto::{NostrPublishRequest, NostrPublishResponse},
    server::BridgeState,
};

pub struct NostrHandler;

impl NostrHandler {
    pub async fn handle(
        state: Arc<BridgeState>,
        request: Request<NostrPublishRequest>,
    ) -> Result<Response<NostrPublishResponse>, Status> {
        let req = request.into_inner();
        info!("📡 PublishNostr: project={}, hash={}", req.project_id, req.design_hash);

        // 1. Verifica se o replicator está ativo
        let replicator = state
            .nostr_replicator
            .as_ref()
            .ok_or_else(|| Status::unavailable("Nostr replicator não configurado"))?;

        // Simplified for mock
        let keys = nostr_sdk::Keys::generate();
        let event = nostr_sdk::EventBuilder::new(
            nostr_sdk::Kind::Custom(30078),
            req.wormgraph_json,
            vec![],
        )
        .to_event(&keys)
        .map_err(|e| Status::internal(e.to_string()))?;

        // 3. Publica nos relays
        let relays = if req.relay_urls.is_empty() {
            replicator.default_relays().to_vec()
        } else {
            req.relay_urls
        };

        let published_event_id = replicator
            .publish_to_relays(&event, &relays)
            .await
            .map_err(|e| Status::internal(format!("Falha na publicação: {}", e)))?;

        info!("✅ Evento publicado: {}", published_event_id.to_hex());

        Ok(Response::new(NostrPublishResponse {
            success: true,
            event_id_hex: published_event_id.to_hex(),
            relay_urls: relays,
            error: None,
            published_at: chrono::Utc::now().timestamp() as u64,
        }))
    }
}
