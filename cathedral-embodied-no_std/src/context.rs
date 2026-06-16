pub struct ContextEmbedding {
    pub calibration_error: f64,
    pub avg_interference: f32,
    pub acceptance_rate: f32,
    pub proof_latency_ms: f64,
    pub memory_proof_usage_rate: f32,
    pub high_risk_action_rate: f32,
    pub recent_audit_flags: u32,
    pub stagnation_rounds: u32,
}
