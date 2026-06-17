//! Cathedral ARKHE v28.3.2 — Módulo de Geometria Causal
//! Implementação do produto interno causal (CIP) e operações geométricas.
//! Selo: CATHEDRAL-ARKHE-v28.3.2-GEOMETRY-MOD-2026-06-16

pub mod causal_inner_product;
pub mod concept_directions;
pub mod steering_vectors;
pub mod subspace_operations;
pub mod embedding_bridge;
pub mod metrics;
pub mod service;

pub use causal_inner_product::{CovarianceMatrix};
pub use concept_directions::{ConceptDirection, ConceptCatalog};
pub use steering_vectors::{SteeringVector, SteeringFactory};
pub use subspace_operations::SubspaceOperations;
pub use embedding_bridge::{EmbeddingModel, EmbeddingBridge};
pub use metrics::GeometryMetrics;
pub use service::CausalGeometryService;
