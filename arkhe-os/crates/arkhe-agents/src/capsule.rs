use arkhe_core::types::AgentId;

pub struct Capsule {
    pub agent_id: AgentId,
}

impl Capsule {
    pub fn new(agent_id: AgentId) -> Self {
        Self { agent_id }
    }
}
