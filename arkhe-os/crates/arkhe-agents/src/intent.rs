use arkhe_core::types::Intent;
use std::collections::VecDeque;

pub struct IntentScheduler {
    queue: VecDeque<Intent>,
}

impl IntentScheduler {
    pub fn new() -> Self {
        Self { queue: VecDeque::new() }
    }

    pub fn submit(&mut self, intent: Intent) {
        self.queue.push_back(intent);
    }

    pub fn schedule_next(&mut self) -> Option<Intent> {
        self.queue.pop_front()
    }
}
