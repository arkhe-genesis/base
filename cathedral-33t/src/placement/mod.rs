//! Expert Placement: Occult + Hybrid-EP

mod occult;
mod hybrid_ep;

pub use occult::{OccultPlacementOptimizer, OccultStats};
pub use hybrid_ep::{HybridEP, CommunicationStrategy};

pub type DeviceId = u32;
pub type ExpertId = usize;
