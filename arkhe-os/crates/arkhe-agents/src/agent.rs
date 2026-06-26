use crate::intent::IntentScheduler; // updated since we don't need Intent directly here if we use IntentScheduler
use arkhe_core::types::{AgentId, CapabilityToken, Intent};
use crate::capsule::Capsule;
use std::collections::HashMap;

pub struct Agent {
    pub id: AgentId,
    pub name: String,
    pub capsule: Capsule,
    pub capabilities: Vec<CapabilityToken>,
    pub intent_queue: Vec<Intent>,
}

impl Agent {
    pub fn new(id: AgentId, name: &str) -> Self {
        Self {
            id,
            name: name.to_string(),
            capsule: Capsule::new(id),
            capabilities: Vec::new(),
            intent_queue: Vec::new(),
        }
    }

    pub fn submit_intent(&mut self, intent: Intent) {
        self.intent_queue.push(intent);
    }
}

pub struct AgentManager {
    agents: std::collections::HashMap<AgentId, Agent>,
    next_id: u64,
}

impl AgentManager {
    pub fn new() -> Self {
        Self { agents: HashMap::new(), next_id: 0 }
    }

    pub fn create_agent(&mut self, name: &str) -> AgentId {
        let id = AgentId(self.next_id);
        self.next_id += 1;
        self.agents.insert(id, Agent::new(id, name));
        id
    }

    pub fn get_agent(&self, id: AgentId) -> Option<&Agent> {
        self.agents.get(&id)
    }
}
