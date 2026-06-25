use std::collections::HashMap;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VectorClock {
    pub entries: HashMap<String, u64>,
}

impl VectorClock {
    pub fn new() -> Self {
        Self { entries: HashMap::new() }
    }

    pub fn increment(&mut self, worker_id: &str) {
        let count = self.entries.entry(worker_id.to_string()).or_insert(0);
        *count += 1;
    }

    pub fn merge(&mut self, other: &VectorClock) {
        for (k, v) in &other.entries {
            let entry = self.entries.entry(k.clone()).or_insert(0);
            if *entry < *v {
                *entry = *v;
            }
        }
    }

    pub fn compare(&self, other: &VectorClock) -> Ordering {
        let mut self_greater = false;
        let mut other_greater = false;

        for (k, v) in &self.entries {
            let other_v = other.entries.get(k).unwrap_or(&0);
            if v > other_v {
                self_greater = true;
            }
            if v < other_v {
                other_greater = true;
            }
        }

        for (k, v) in &other.entries {
            if !self.entries.contains_key(k) {
                other_greater = true;
            }
        }

        if self_greater && !other_greater {
            Ordering::Greater
        } else if other_greater && !self_greater {
            Ordering::Less
        } else if !self_greater && !other_greater {
            Ordering::Equal
        } else {
            Ordering::Concurrent
        }
    }
}

#[derive(Debug, PartialEq)]
pub enum Ordering {
    Greater,
    Less,
    Equal,
    Concurrent,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EpisodicEntry {
    pub id: String,
    pub user_input: String,
    pub assistant_output: String,
    pub timestamp: i64,
    pub version: u64,
    pub vector_clock: VectorClock,
    pub worker_id: String,
    pub confidence: f32,
    pub deleted: bool,
}
