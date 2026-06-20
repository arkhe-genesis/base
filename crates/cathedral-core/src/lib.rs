//! Cathedral Core — Facade de integração
//! Selo: CATHEDRAL-ARKHE-CORE-v1.0.0-2026-06-19

// Reexporta todos os crates
pub use cathedral_agi as agi;
pub use cathedral_agi::{
    AGICore, EthicsVerifier, HierarchicalWormhole, MCTSEngine, MetaCognitiveLoop, OllamaClient,
    WorldModel,
};
pub use cathedral_edge_agent as edge;
pub use cathedral_episodic as episodic;
pub use cathedral_episodic::{EpisodicEntry, EpisodicSync, Ordering, VectorClock};
pub use cathedral_fallback as fallback;
pub use cathedral_fallback::{CostOptimizer, FallbackChain, OptimizationStats};
pub use cathedral_scheduler as scheduler;
// Conveniência: reexporta tipos comuns
pub use cathedral_scheduler::{
    HybridScheduler, SchedulerStats, SchedulingDecision, TaskType, TeeType, WorkerProfile,
    WorkerRegistry, WorkerTier,
};
pub use cathedral_tee as tee;
pub use cathedral_tee::{AttestationReport, AttestationResult, TEEBridge};

// Versão unificada
pub const VERSION: &str = env!("CARGO_PKG_VERSION");
