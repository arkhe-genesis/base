//! FallbackChain + CostOptimizer
//! Selo: CATHEDRAL-ARKHE-FALLBACK-v1.0.0-2026-06-19

mod fallback;
mod cost;

pub use fallback::{FallbackChain, TaskExecutor, WorkerExecutor, WorkerTier};
pub use cost::{CostOptimizer, CostRecord, OptimizationStats};
