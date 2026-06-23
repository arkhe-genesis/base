//! Cathedral ARKHE 33T v4.0 — ASI Architecture
//!
//! Módulos principais:
//! - `moe`: Mixture of Experts (4096 experts)
//! - `attention`: CSA + HCA + MLA
//! - `mhc`: Manifold-Constrained Hyper-Connections
//! - `optimizer`: MONA-Lite
//! - `routing`: Anticipatory Routing
//! - `speculative`: EAGLE-3
//! - `placement`: Occult + Hybrid-EP
//! - `ssm`: SSM Engine (Mamba‑2)
//! - `symbolic`: Neuro‑Symbolic Reasoner
//! - `swarm`: Swarm Coordination Layer
//! - `consistency`: Behavioral Consistency Engine
//! - `platform`: Deteção de plataforma

pub mod config;
pub mod tensor;
pub mod moe;
pub mod attention;
pub mod mhc;
pub mod optimizer;
pub mod routing;
pub mod speculative;
pub mod placement;
pub mod utils;
pub mod platform;

#[cfg(feature = "ssm")]
pub mod ssm;

#[cfg(feature = "symbolic")]
pub mod symbolic;

#[cfg(feature = "swarm")]
pub mod swarm;

#[cfg(feature = "consistency")]
pub mod consistency;

// Re-export principais
pub use config::CathedralConfig;
pub use tensor::Tensor;
pub use moe::MoELayer;
pub use attention::HybridAttention;
pub use mhc::ManifoldConstrainedHyperConnections;
pub use optimizer::MONALiteOptimizer;
pub use routing::AnticipatoryRouter;
pub use speculative::Eagle3Decoder;
pub use placement::{OccultPlacementOptimizer, HybridEP};
pub use platform::{Platform, init_platform};

pub const VERSION: &str = env!("CARGO_PKG_VERSION");
