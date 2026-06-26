use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum TaskType {
    Inference,
    MCTS,
    Compression,
    WorldModel,
    EthicsCheck,
    MemorySync,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum WorkerTier {
    Datacenter,
    DePinGpu,
    DePinCpu,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum TeeType {
    SGX,
    SevSnp,
    IoNet,
    None,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkerProfile {
    pub worker_id: String,
    pub tier: WorkerTier,
    pub endpoint: String,
    pub cost_per_hour: f64,
    pub latency_p50_ms: u64,
    pub latency_p95_ms: u64,
    pub reputation: f32,
    pub stake_sats: u64,
    pub last_attestation: i64,
    pub tasks_completed: u64,
    pub tasks_failed: u64,
    pub available: bool,
    pub tee_type: TeeType,
    pub capabilities: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SchedulingDecision {
    pub task_id: String,
    pub task_type: TaskType,
    pub selected_worker: String,
    pub selected_tier: WorkerTier,
    pub estimated_cost: f64,
    pub estimated_latency_ms: u64,
    pub reason: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SchedulerStats {
    pub total_workers: usize,
    pub depin_gpu_count: usize,
    pub depin_cpu_count: usize,
    pub datacenter_count: usize,
    pub avg_reputation: f32,
    pub active_workers: usize,
}
