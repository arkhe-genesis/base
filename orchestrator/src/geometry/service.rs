//! Cathedral ARKHE v28.3.2 — CausalGeometryService
//! Camada unificada para toda a geometria causal.
//! Selo: CATHEDRAL-ARKHE-v28.3.2-GEOMETRY-SERVICE-2026-06-16

use std::sync::Arc;
use ndarray::{Array1, ArrayView1};

use super::causal_inner_product::CovarianceMatrix;
use super::concept_directions::ConceptCatalog;
use super::steering_vectors::SteeringFactory;
use super::subspace_operations::SubspaceOperations;

pub struct CausalGeometryService {
    cov: Arc<CovarianceMatrix>,
    subspace_ops: Arc<SubspaceOperations>,
}

impl CausalGeometryService {
    pub fn new(embedding_dim: usize) -> Self {
        let cov = Arc::new(CovarianceMatrix::identity(embedding_dim));
        let subspace_ops = Arc::new(SubspaceOperations::new(cov.clone()));
        Self { cov, subspace_ops }
    }
}
