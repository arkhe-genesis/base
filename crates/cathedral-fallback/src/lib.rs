//! FallbackChain + CostOptimizer
//! Selo: CATHEDRAL-ARKHE-FALLBACK-v1.0.0-2026-06-19

mod cost;
mod fallback;

pub use cost::{CostOptimizer, CostRecord, OptimizationStats};
pub use fallback::{FallbackChain, TaskExecutor, WorkerExecutor, WorkerTier};
