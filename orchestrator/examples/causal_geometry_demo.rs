//! Cathedral ARKHE v28.3.2 — Causal Geometry Demo
//! Demonstra a camada de geometria causal em ação.
//!
//! Execute com: cargo run --example causal_geometry_demo

use ndarray::Array1;
use orchestrator::geometry::embedding_bridge::EmbeddingModel;
use orchestrator::geometry::service::CausalGeometryService;
use orchestrator::governance::geometric_policy_engine::GeometricPolicyEngine;
use orchestrator::orchestrator::AgentRole;
use std::sync::Arc;

pub struct SimpleEmbedder {
    dim: usize,
}

impl SimpleEmbedder {
    pub fn new(dim: usize) -> Self {
        Self { dim }
    }
}

impl EmbeddingModel for SimpleEmbedder {
    fn embed(&self, _text: &str) -> Array1<f32> {
        Array1::zeros(self.dim)
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 1. Inicializa serviço de geometria
    let embedder = Arc::new(SimpleEmbedder::new(768));
    let geometry = Arc::new(CausalGeometryService::new(embedder, 768));

    let mut code_emb = Array1::zeros(768);
    code_emb[0] = 1.0;
    let mut non_code_emb = Array1::zeros(768);
    non_code_emb[1] = 1.0;
    let mut safe_emb = Array1::zeros(768);
    safe_emb[2] = 1.0;
    let mut unsafe_emb = Array1::zeros(768);
    unsafe_emb[3] = 1.0;
    let mut mem_emb = Array1::zeros(768);
    mem_emb[4] = 1.0;
    let mut non_mem_emb = Array1::zeros(768);
    non_mem_emb[5] = 1.0;

    // 2. Registra conceitos
    geometry.register_concept("code", &[code_emb], &[non_code_emb]).await?;
    geometry.register_concept("safety", &[safe_emb], &[unsafe_emb]).await?;
    geometry.register_concept("memory", &[mem_emb.clone()], &[non_mem_emb.clone()]).await?;
    geometry.register_concept("memory_efficient", &[mem_emb], &[non_mem_emb]).await?;

    // 3. Gera steering para "memory_efficient"
    let _steering = geometry.get_steering_vector("memory_efficient", 0.5).await?;

    // 4. Mede ortogonalidade
    let orth = geometry.concept_orthogonality("code", "safety").await.unwrap_or(0.0);
    println!("Ortogonalidade code-safety: {:.3}", orth);

    // 5. Exemplo de política geométrica
    let policy_engine = GeometricPolicyEngine::new(geometry.clone());
    let result = policy_engine
        .authorize(AgentRole::Specialist, "generate_kernel", "cuda_kernel_code", None, None)
        .await;

    match result {
        Ok(()) => println!("✅ Ação autorizada"),
        Err(e) => println!("❌ Ação rejeitada: {}", e),
    }

    Ok(())
}
