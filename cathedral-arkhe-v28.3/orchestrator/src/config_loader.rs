//! Carregador de configuração do agente a partir de arquivos YAML/JSON.
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct AgentConfigFile {
    pub agent: AgentSection,
    pub governance: GovernanceSection,
    pub telemetry: TelemetrySection,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct AgentSection {
    pub id: String,
    pub role: String,
    pub version: String,
    pub system_prompt_path: String,
    pub tools_registry: String,
    pub planning: PlanningConfig,
    pub memory: MemoryConfig,
    pub trust: TrustConfig,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct PlanningConfig {
    pub strategy: String,
    pub max_steps: u32,
    pub consensus_mode: String,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct MemoryConfig {
    pub short_term_capacity: usize,
    pub long_term_enabled: bool,
    pub vector_db: String,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct TrustConfig {
    pub require_memory_proof: bool,
    pub require_spex: bool,
    pub post_quantum_signature: bool,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct GovernanceSection {
    pub constitution: String,
    pub policy_hash: String,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct TelemetrySection {
    pub otel_endpoint: String,
    pub log_level: String,
}

impl AgentConfigFile {
    pub fn from_yaml<P: AsRef<Path>>(path: P) -> Result<Self, String> {
        let content = fs::read_to_string(path).map_err(|e| e.to_string())?;
        serde_yaml::from_str(&content).map_err(|e| e.to_string())
    }
}