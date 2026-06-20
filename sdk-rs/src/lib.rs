pub mod grpc {
    tonic::include_proto!("cathedral.v1");
}

use std::{collections::HashMap, time::Duration};

use anyhow::{Result, anyhow};
use grpc::{
    Event, EventMetadata, EventType, GovernanceRequest, GovernanceVerdict, IngestRequest,
    cathedral_bridge_client::CathedralBridgeClient,
};
use serde::{Deserialize, Serialize};
use tokio::sync::mpsc;
use tracing::{debug, error};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "event_type")]
pub enum SdkEvent {
    DesignProposed {
        design_hash: String,
        parent_hashes: Vec<String>,
        parameters: HashMap<String, f64>,
        rationale: String,
        agent_id: String,
    },
    SimulationCompleted {
        design_hash: String,
        simulator: String,
        metrics: HashMap<String, f64>,
        convergence: bool,
        compute_cost_usd: f64,
    },
    AgentMutation {
        mutation_description: String,
        previous_agent_hash: String,
        substrate_version: String,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GovernanceResponseResult {
    pub verdict: String,
    pub rationale: String,
    pub conditions: Option<Vec<String>>,
}

#[derive(Debug, Clone)]
pub struct SdkConfig {
    pub bridge_endpoint: String,
    pub project_id: String,
    pub agent_id: String,
    pub batch_size: usize,
    pub flush_interval_ms: u64,
    pub governance_mode: GovernanceMode,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GovernanceMode {
    HumanInTheLoop,
    AutonomousWithCircuitBreaker,
    AutonomousFull,
}

impl Default for SdkConfig {
    fn default() -> Self {
        Self {
            bridge_endpoint: "http://[::1]:50051".to_string(),
            project_id: "default".to_string(),
            agent_id: "default-agent".to_string(),
            batch_size: 50,
            flush_interval_ms: 5000,
            governance_mode: GovernanceMode::AutonomousWithCircuitBreaker,
        }
    }
}

pub struct CathedralSdk {
    pub config: SdkConfig,
    event_tx: mpsc::UnboundedSender<SdkEvent>,
    pub metrics: SdkMetrics,
}

#[derive(Debug, Clone, Default)]
pub struct SdkMetrics {
    pub events_emitted: u64,
    pub events_batched: u64,
}

impl CathedralSdk {
    pub async fn new(config: SdkConfig) -> Result<Self> {
        let (tx, mut rx) = mpsc::unbounded_channel();

        // Check if endpoint parses correctly
        let endpoint = tonic::transport::Endpoint::from_shared(config.bridge_endpoint.clone())?;
        let channel = endpoint.connect_timeout(Duration::from_secs(5)).connect().await?;
        let client = CathedralBridgeClient::new(channel);

        let config_clone = config.clone();
        tokio::spawn(async move {
            let mut batch = Vec::with_capacity(config_clone.batch_size);
            let mut interval =
                tokio::time::interval(Duration::from_millis(config_clone.flush_interval_ms));
            let mut client = client;

            loop {
                tokio::select! {
                    _ = interval.tick() => {
                        if !batch.is_empty() {
                            Self::flush_batch(&mut client, &config_clone, &mut batch).await;
                        }
                    }
                    Some(event) = rx.recv() => {
                        batch.push(event);
                        if batch.len() >= config_clone.batch_size {
                            Self::flush_batch(&mut client, &config_clone, &mut batch).await;
                            interval.reset(); // Reset interval since we just flushed
                        }
                    }
                    else => break,
                }
            }
            if !batch.is_empty() {
                Self::flush_batch(&mut client, &config_clone, &mut batch).await;
            }
        });

        Ok(Self { config, event_tx: tx, metrics: SdkMetrics::default() })
    }

    async fn flush_batch(
        client: &mut CathedralBridgeClient<tonic::transport::Channel>,
        config: &SdkConfig,
        batch: &mut Vec<SdkEvent>,
    ) {
        if batch.is_empty() {
            return;
        }

        let mut grpc_events = vec![];
        for event in batch.iter() {
            let (event_type, design_hash, parent_hashes, payload, domain, cost) = match event {
                SdkEvent::DesignProposed {
                    design_hash,
                    parent_hashes,
                    parameters,
                    rationale: _,
                    agent_id: _,
                } => (
                    EventType::DesignProposed as i32,
                    design_hash.clone(),
                    parent_hashes.clone(),
                    serde_json::to_string(&parameters).unwrap(),
                    "aerospace".to_string(),
                    0.0,
                ),
                SdkEvent::SimulationCompleted {
                    design_hash,
                    simulator: _,
                    metrics,
                    convergence: _,
                    compute_cost_usd,
                } => (
                    EventType::SimulationCompleted as i32,
                    design_hash.clone(),
                    vec![],
                    serde_json::to_string(&metrics).unwrap(),
                    "simulation".to_string(),
                    *compute_cost_usd,
                ),
                SdkEvent::AgentMutation {
                    mutation_description,
                    previous_agent_hash,
                    substrate_version: _,
                } => (
                    EventType::AgentMutation as i32,
                    blake3::hash(mutation_description.as_bytes()).to_hex().to_string(),
                    vec![previous_agent_hash.clone()],
                    serde_json::to_string(&mutation_description).unwrap(),
                    "meta".to_string(),
                    0.0,
                ),
            };

            grpc_events.push(Event {
                event_id: uuid::Uuid::new_v4().to_string(),
                timestamp: Some(prost_types::Timestamp {
                    seconds: std::time::SystemTime::now()
                        .duration_since(std::time::UNIX_EPOCH)
                        .unwrap()
                        .as_secs() as i64,
                    nanos: 0,
                }),
                event_type,
                design_hash,
                parent_hashes,
                payload_json: payload,
                metadata: Some(EventMetadata {
                    domain,
                    confidence: 0.8,
                    compute_cost_usd: cost,
                    tags: vec![],
                }),
            });
        }

        let request = tonic::Request::new(IngestRequest {
            project_id: config.project_id.clone(),
            agent_id: config.agent_id.clone(),
            events: grpc_events,
            batch_id: Some(uuid::Uuid::new_v4().to_string()),
        });

        match client.ingest(request).await {
            Ok(resp) => {
                debug!(
                    "✅ Batch of {} events sent successfully. Bridge response: {}",
                    batch.len(),
                    resp.into_inner().message
                );
            }
            Err(e) => {
                error!("❌ Failed to send batch via gRPC: {}", e);
            }
        }
        batch.clear();
    }

    pub async fn emit_design_proposed(
        &self,
        design_hash: String,
        parent_hashes: Vec<String>,
        parameters: HashMap<String, f64>,
        rationale: String,
    ) -> Result<()> {
        let event = SdkEvent::DesignProposed {
            design_hash,
            parent_hashes,
            parameters,
            rationale,
            agent_id: self.config.agent_id.clone(),
        };
        self.event_tx
            .send(event)
            .map_err(|e| anyhow!("Failed to send event to background task: {}", e))?;
        Ok(())
    }

    pub async fn emit_simulation_completed(
        &self,
        design_hash: String,
        simulator: String,
        metrics: HashMap<String, f64>,
        convergence: bool,
        compute_cost_usd: f64,
    ) -> Result<()> {
        let event = SdkEvent::SimulationCompleted {
            design_hash,
            simulator,
            metrics,
            convergence,
            compute_cost_usd,
        };
        self.event_tx
            .send(event)
            .map_err(|e| anyhow!("Failed to send event to background task: {}", e))?;
        Ok(())
    }

    pub async fn request_governance(&self, event: SdkEvent) -> Result<GovernanceResponseResult> {
        if self.config.governance_mode == GovernanceMode::AutonomousFull {
            return Ok(GovernanceResponseResult {
                verdict: "approved".to_string(),
                rationale: "Autonomous full mode".to_string(),
                conditions: None,
            });
        }

        let risk = Self::estimate_risk(&event);
        if self.config.governance_mode == GovernanceMode::AutonomousWithCircuitBreaker && risk < 0.5
        {
            return Ok(GovernanceResponseResult {
                verdict: "approved".to_string(),
                rationale: "Low risk decision".to_string(),
                conditions: None,
            });
        }

        let endpoint =
            tonic::transport::Endpoint::from_shared(self.config.bridge_endpoint.clone())?;
        let channel = endpoint.connect_timeout(Duration::from_secs(5)).connect().await?;
        let mut client = CathedralBridgeClient::new(channel);

        let event_type = match event {
            SdkEvent::DesignProposed { .. } => EventType::DesignProposed,
            SdkEvent::SimulationCompleted { .. } => EventType::SimulationCompleted,
            SdkEvent::AgentMutation { .. } => EventType::AgentMutation,
        };

        let payload_json = serde_json::to_string(&event)?;

        let request = tonic::Request::new(GovernanceRequest {
            request_id: uuid::Uuid::new_v4().to_string(),
            project_id: self.config.project_id.clone(),
            agent_id: self.config.agent_id.clone(),
            event_type: event_type as i32,
            proposed_state_json: payload_json,
            current_state_json: "{}".to_string(),
            agent_risk_score: risk,
            domain: "aerospace".to_string(),
            metadata: HashMap::new(),
        });

        let response = client.request_governance(request).await?;
        let resp_inner = response.into_inner();

        let verdict_str = match GovernanceVerdict::try_from(resp_inner.verdict) {
            Ok(GovernanceVerdict::Approved) => "approved",
            Ok(GovernanceVerdict::Rejected) => "rejected",
            Ok(GovernanceVerdict::RequiresHuman) => "requires_human",
            Ok(GovernanceVerdict::Conditional) => "conditional",
            Ok(GovernanceVerdict::Timeout) => "timeout",
            _ => "unknown",
        };

        Ok(GovernanceResponseResult {
            verdict: verdict_str.to_string(),
            rationale: resp_inner.rationale,
            conditions: if resp_inner.conditions.is_empty() {
                None
            } else {
                Some(resp_inner.conditions)
            },
        })
    }

    fn estimate_risk(event: &SdkEvent) -> f64 {
        match event {
            SdkEvent::AgentMutation { .. } => 0.85,
            SdkEvent::DesignProposed { .. } => 0.20,
            SdkEvent::SimulationCompleted { .. } => 0.30,
        }
    }
}
