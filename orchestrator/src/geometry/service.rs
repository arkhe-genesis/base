//! Cathedral ARKHE v28.3.2 — CausalGeometryService
//! Camada unificada para toda a geometria causal.
//! Selo: CATHEDRAL-ARKHE-v28.3.2-GEOMETRY-SERVICE-2026-06-16

use std::sync::Arc;

use ndarray::{Array1, ArrayView1};
use tokio::sync::RwLock;

use super::{
    causal_inner_product::CovarianceMatrix, concept_directions::ConceptCatalog,
    embedding_bridge::EmbeddingModel, steering_vectors::SteeringFactory,
    subspace_operations::SubspaceOperations,
};

pub struct CausalGeometryService {
    cov: Arc<CovarianceMatrix>,
    catalog: Arc<RwLock<ConceptCatalog>>,
    steering_factory: Arc<RwLock<SteeringFactory>>,
    subspace_ops: Arc<SubspaceOperations>,
    embedder: Arc<dyn EmbeddingModel + Send + Sync>,
}

impl CausalGeometryService {
    pub fn new(embedder: Arc<dyn EmbeddingModel + Send + Sync>, embedding_dim: usize) -> Self {
        let cov = Arc::new(CovarianceMatrix::identity(embedding_dim));
        let catalog = Arc::new(RwLock::new(ConceptCatalog::new(cov.clone())));
        let steering_factory = Arc::new(RwLock::new(SteeringFactory::new(
            (*cov).clone(),
            ConceptCatalog::new(cov.clone()),
        )));
        let subspace_ops = Arc::new(SubspaceOperations::new(cov.clone()));

        Self { cov, catalog, steering_factory, subspace_ops, embedder }
    }

    /// Embedding de texto para vetor
    pub fn embed(&self, text: &str) -> Array1<f32> {
        self.embedder.embed(text)
    }

    /// Registra um conceito a partir de exemplos
    pub async fn register_concept(
        &self,
        name: &str,
        positive: &[Array1<f32>],
        negative: &[Array1<f32>],
    ) -> Result<(), String> {
        self.catalog.write().await.register_concept(name, positive, negative)?;

        let mut factory = self.steering_factory.write().await;
        // update the catalog in the factory as well
        *factory = SteeringFactory::new((*self.cov).clone(), (*self.catalog.read().await).clone());

        Ok(())
    }

    /// Obtém a direção de um conceito
    pub async fn get_concept_direction(&self, name: &str) -> Option<Array1<f32>> {
        self.catalog.read().await.get_direction(name)
    }

    /// Ortogonalidade entre dois conceitos
    pub async fn concept_orthogonality(&self, a: &str, b: &str) -> Option<f32> {
        self.catalog.read().await.orthogonality(a, b)
    }

    /// Gera steering vector para um conceito
    pub async fn get_steering_vector(
        &self,
        concept: &str,
        intensity: f32,
    ) -> Result<Array1<f32>, String> {
        self.steering_factory.write().await.get_steering_vector(concept, intensity)
    }

    /// Gera steering ortogonal a conceitos indesejados
    pub async fn get_orthogonal_steering(
        &self,
        concept: &str,
        avoid: &[&str],
        intensity: f32,
    ) -> Result<Array1<f32>, String> {
        self.steering_factory.write().await.get_orthogonal_steering(concept, avoid, intensity)
    }

    /// Produto interno causal
    pub fn causal_dot(&self, a: &ArrayView1<f32>, b: &ArrayView1<f32>) -> f32 {
        self.cov.causal_dot(a, b)
    }

    /// Similaridade causal
    pub fn causal_similarity(&self, a: &ArrayView1<f32>, b: &ArrayView1<f32>) -> f32 {
        self.cov.causal_similarity(a, b)
    }

    /// Ortogonalidade causal
    pub fn causal_orthogonality(&self, a: &ArrayView1<f32>, b: &ArrayView1<f32>) -> f32 {
        self.cov.causal_orthogonality(a, b)
    }

    /// Projeção causal
    pub fn causal_project(&self, v: &ArrayView1<f32>, u: &ArrayView1<f32>) -> Array1<f32> {
        self.cov.causal_project(v, u)
    }

    /// Rank causal aproximado
    pub fn causal_rank(&self, _v: &ArrayView1<f32>) -> usize {
        // Em produção: SVD na projeção causal
        // Placeholder: 4
        4
    }

    /// Projeta um embedding no subespaço causal de conceitos conhecidos
    pub fn project_causal(&self, v: &Array1<f32>) -> Array1<f32> {
        self.subspace_ops.project_to_known_subspace(v)
    }

    /// Peso causal de um vetor (importância conceitual)
    pub fn causal_weight(&self, v: &Array1<f32>) -> f32 {
        self.subspace_ops.causal_weight(v)
    }
}
