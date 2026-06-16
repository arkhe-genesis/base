use std::fs;
use crate::config_loader::AgentConfigFile;

pub struct OrchestratorError(pub String);

impl OrchestratorError {
    pub fn invalid_task(msg: String) -> Self {
        OrchestratorError(msg)
    }
}

pub struct Agent {
    pub id: String,
    pub role: String,
    pub strategy: String,
    pub require_memory_proof: bool,
}

pub struct MultiAgentOrchestrator {
    pub memory_config: crate::config_loader::MemoryConfig,
    pub trust_config: crate::config_loader::TrustConfig,
    pub planning_config: crate::config_loader::PlanningConfig,
    pub registered_agents: Vec<Agent>,
}

impl MultiAgentOrchestrator {
    pub fn new() -> Self {
        MultiAgentOrchestrator {
            memory_config: crate::config_loader::MemoryConfig {
                short_term_capacity: 0,
                long_term_enabled: false,
                vector_db: String::new(),
            },
            trust_config: crate::config_loader::TrustConfig {
                require_memory_proof: false,
                require_spex: false,
                post_quantum_signature: false,
            },
            planning_config: crate::config_loader::PlanningConfig {
                strategy: String::new(),
                max_steps: 0,
                consensus_mode: String::new(),
            },
            registered_agents: Vec::new(),
        }
    }

    pub fn register_agent(&mut self, agent: Agent) {
        self.registered_agents.push(agent);
    }

    pub async fn from_config_files(
        config_path: &str,
        manifest_path: &str,
    ) -> Result<Self, OrchestratorError> {
        // 1. Carregar config.yaml
        let agent_config = AgentConfigFile::from_yaml(config_path)
            .map_err(|e| OrchestratorError::invalid_task(format!("Config load error: {}", e)))?;

        // 2. Carregar manifest.json
        let manifest_content = fs::read_to_string(manifest_path)
            .map_err(|e| OrchestratorError::invalid_task(e.to_string()))?;
        let _manifest: serde_json::Value = serde_json::from_str(&manifest_content)
            .map_err(|e| OrchestratorError::invalid_task(e.to_string()))?;

        // 3. Configurar memória e ferramentas a partir do config
        let memory_cfg = agent_config.agent.memory.clone();
        let trust_cfg = agent_config.agent.trust.clone();
        let planning_cfg = agent_config.agent.planning.clone();

        // 4. Inicializar orquestrador com esses parâmetros
        let mut orchestrator = Self::new();
        orchestrator.memory_config = memory_cfg;
        orchestrator.trust_config = trust_cfg;
        orchestrator.planning_config = planning_cfg;

        let agent = Agent {
            id: agent_config.agent.id,
            role: agent_config.agent.role,
            strategy: agent_config.agent.planning.strategy,
            require_memory_proof: agent_config.agent.trust.require_memory_proof,
        };

        orchestrator.register_agent(agent);

        Ok(orchestrator)
    }
}
