//! Cathedral Core — Facade de integração
//! Selo: CATHEDRAL-ARKHE-CORE-v1.0.0-2026-06-19

// Reexporta todos os crates
pub use cathedral_scheduler as scheduler;
pub use cathedral_episodic as episodic;
pub use cathedral_tee as tee;
pub use cathedral_fallback as fallback;
pub use cathedral_agi as agi;
pub use cathedral_edge_agent as edge;

// Conveniência: reexporta tipos comuns
pub use cathedral_scheduler::{
    HybridScheduler, WorkerRegistry, WorkerProfile, WorkerTier, TaskType, SchedulingDecision, SchedulerStats, TeeType,
};
pub use cathedral_episodic::{EpisodicSync, EpisodicEntry, VectorClock, Ordering};
pub use cathedral_tee::{TEEBridge, AttestationReport, AttestationResult};
pub use cathedral_fallback::{FallbackChain, CostOptimizer, OptimizationStats};
pub use cathedral_agi::{AGICore, OllamaClient, WorldModel, MCTSEngine, MetaCognitiveLoop, HierarchicalWormhole, EthicsVerifier};

// Versão unificada
pub const VERSION: &str = env!("CARGO_PKG_VERSION");
