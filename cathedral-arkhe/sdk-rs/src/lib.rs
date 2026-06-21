mod agent_tree;
mod grpc_client;

pub use agent_tree::{AgentTree, AgentTreeNode};
pub use grpc_client::GrpcClient;

use anyhow::{bail, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::Duration;
use zstd::encode_all;
use std::io::Cursor;
use tracing::{debug, error, warn};

pub mod cathedral {
    pub mod v1 {
        tonic::include_proto!("cathedral.v1");
    }
}

use cathedral::v1::{Event, EventType, EventMetadata};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentIdentity {
    pub agent_id: String,
    pub parent_agent_id: Option<String>,
    pub tree_id: Option<String>,
    pub subagent_ids: Vec<String>,
    pub role: String,
    pub depth: u32,
    pub reputation_hash: Option<String>,
    pub metadata: HashMap<String, String>,
}

impl AgentIdentity {
    pub fn new(agent_id: &str, role: &str) -> Self {
        Self {
            agent_id: agent_id.to_string(),
            parent_agent_id: None,
            tree_id: None,
            subagent_ids: Vec::new(),
            role: role.to_string(),
            depth: 0,
            reputation_hash: None,
            metadata: HashMap::new(),
        }
    }
    pub fn with_parent(mut self, parent: &str) -> Self { self.parent_agent_id = Some(parent.to_string()); self.depth = 1; self }
    pub fn with_tree(mut self, tree: &str) -> Self { self.tree_id = Some(tree.to_string()); self }
}

#[derive(Debug, Clone, Default)]
pub struct SdkMetrics {
    pub avg_latency_ms: f64,
    pub bytes_sent: u64,
    pub events_emitted: u64,
    pub events_failed: u64,
    pub events_retried: u64,
}

#[derive(Debug, Clone)]
pub struct CathedralSdkConfig {
    pub bridge_endpoint: String,
    pub project_id: String,
    pub agent_id: String,
    pub initial_tree: Option<AgentTree>,
    pub compression_enabled: bool,
    pub max_retries: u32,
    pub local_logging_enabled: bool,
    pub mode: SdkMode,
}

#[derive(Debug, Clone, PartialEq, Default)]
pub enum SdkMode {
    #[default]
    Async,
    Realtime,
}

impl Default for CathedralSdkConfig {
    fn default() -> Self {
        Self {
            bridge_endpoint: "grpc://localhost:9002".to_string(),
            project_id: "default".to_string(),
            agent_id: "default-agent".to_string(),
            initial_tree: None,
            compression_enabled: true,
            max_retries: 3,
            local_logging_enabled: false,
            mode: Default::default(),
        }
    }
}

pub struct CathedralSdk {
    pub config: CathedralSdkConfig,
    _agent_tree: Option<AgentTree>,
    _grpc_client: GrpcClient,
    pub client: reqwest::Client,
    pub metrics: SdkMetrics,
}

impl CathedralSdk {
    pub async fn new(config: CathedralSdkConfig) -> Result<Self> {
        let grpc_client = GrpcClient::connect(&config.bridge_endpoint).await?;
        let agent_tree = config.initial_tree.clone();
        Ok(Self { config, _agent_tree: agent_tree, _grpc_client: grpc_client, client: reqwest::Client::new(), metrics: SdkMetrics::default() })
    }

    pub async fn emit_design_proposed(
        &self,
        event_id: String,
        parent_hashes: Vec<String>,
        payload: serde_json::Value,
        _rationale: String,
    ) -> Result<()> {
        let event = Event {
            event_id,
            timestamp: Some(prost_types::Timestamp { seconds: chrono::Utc::now().timestamp(), nanos: 0 }),
            event_type: EventType::DesignProposed as i32,
            design_hash: "".to_string(),
            parent_hashes,
            payload_json: payload.to_string(),
            metadata: Some(EventMetadata {
                domain: "default".to_string(),
                confidence: 1.0,
                compute_cost_usd: 0.0,
                tags: vec![],
            }),
            zk_proof: None,
            agent_identity: None,
        };
        self.send_immediately(event).await
    }

    pub async fn emit_simulation_completed(
        &self,
        design_hash: String,
        _simulator: String,
        _metrics: HashMap<String, f64>,
        _success: bool,
        _cost: f64,
    ) -> Result<()> {
        let event = Event {
            event_id: format!("sim-{}", uuid::Uuid::new_v4()),
            timestamp: Some(prost_types::Timestamp { seconds: chrono::Utc::now().timestamp(), nanos: 0 }),
            event_type: EventType::SimulationCompleted as i32,
            design_hash,
            parent_hashes: vec![],
            payload_json: "{}".to_string(),
            metadata: Some(EventMetadata {
                domain: "default".to_string(),
                confidence: 1.0,
                compute_cost_usd: 0.0,
                tags: vec![],
            }),
            zk_proof: None,
            agent_identity: None,
        };
        self.send_immediately(event).await
    }

    pub async fn emit_agent_mutation(
        &self,
        _rationale: String,
        _new_version: String,
        mutation_type: String,
    ) -> Result<()> {
        let event = Event {
            event_id: format!("mut-{}", uuid::Uuid::new_v4()),
            timestamp: Some(prost_types::Timestamp { seconds: chrono::Utc::now().timestamp(), nanos: 0 }),
            event_type: EventType::AgentMutation as i32,
            design_hash: "".to_string(),
            parent_hashes: vec![],
            payload_json: serde_json::json!({
                "mutation_type": mutation_type,
                "agent_id": self.config.agent_id,
                "timestamp": chrono::Utc::now().timestamp(),
            }).to_string(),
            metadata: Some(EventMetadata {
                domain: "default".to_string(),
                confidence: 1.0,
                compute_cost_usd: 0.0,
                tags: vec![],
            }),
            zk_proof: None,
            agent_identity: None,
        };
        self.send_immediately(event).await
    }

    pub async fn emit_parameter_change(
        &self,
        event_id: String,
        payload: serde_json::Value,
        _agent_id: String,
    ) -> Result<()> {
        let event = Event {
            event_id,
            timestamp: Some(prost_types::Timestamp { seconds: chrono::Utc::now().timestamp(), nanos: 0 }),
            event_type: EventType::ParameterChange as i32,
            design_hash: "".to_string(),
            parent_hashes: vec![],
            payload_json: payload.to_string(),
            metadata: Some(EventMetadata {
                domain: "default".to_string(),
                confidence: 1.0,
                compute_cost_usd: 0.0,
                tags: vec![],
            }),
            zk_proof: None,
            agent_identity: None,
        };
        self.send_immediately(event).await
    }

    async fn send_immediately(&self, event: Event) -> Result<()> {
        let payload = if self.config.compression_enabled {
            let mut event_bytes = Vec::new();
            prost::Message::encode(&event, &mut event_bytes)?;
            encode_all(Cursor::new(event_bytes), 3)?
        } else {
            let mut event_bytes = Vec::new();
            prost::Message::encode(&event, &mut event_bytes)?;
            event_bytes
        };

        self.send_with_retry(&payload).await
    }

    async fn send_with_retry(&self, payload: &[u8]) -> Result<()> {
        let url = format!("{}/cathedral.v1.CathedralBridge/Ingest", self.config.bridge_endpoint.replace("grpc://", "http://"));
        let content_type = "application/grpc";

        for attempt in 0..=self.config.max_retries {
            match self.client.post(&url)
                .header("Content-Type", content_type)
                .body(payload.to_vec())
                .send()
                .await
            {
                Ok(response) if response.status().is_success() => return Ok(()),
                Ok(response) => {
                    warn!("HTTP {} on attempt {}", response.status(), attempt);
                }
                Err(e) => {
                    warn!("Request failed on attempt {}: {}", attempt, e);
                }
            }

            if attempt < self.config.max_retries {
                tokio::time::sleep(Duration::from_millis(100 * (attempt + 1) as u64)).await;
            }
        }

        bail!("Max retries exceeded")
    }
}
