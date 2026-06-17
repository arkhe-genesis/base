//! Cathedral ARKHE v28.3.2 — Geometric Cuda Reward Model
//! Combina speedup com qualidade geométrica (ortogonalidade + rank causal).
//! Selo: CATHEDRAL-ARKHE-v28.3.2-GEOMETRIC-REWARD-2026-06-16

use std::sync::Arc;

use tracing::debug;

use crate::{cuda::CudaRewardModel, geometry::service::CausalGeometryService};

pub struct GeometricCudaReward {
    base_evaluator: Arc<CudaRewardModel>,
    geometry: Arc<CausalGeometryService>,
    /// Peso da componente geométrica (0..1)
    geometric_weight: f32,
    /// Conceitos alvo para steering (ex: "memory_efficient")
    target_concepts: Vec<String>,
    /// Conceitos a evitar (ex: "bug_prone")
    avoid_concepts: Vec<String>,
}

impl GeometricCudaReward {
    pub fn new(
        base_evaluator: Arc<CudaRewardModel>,
        geometry: Arc<CausalGeometryService>,
        geometric_weight: f32,
        target_concepts: Vec<String>,
        avoid_concepts: Vec<String>,
    ) -> Self {
        Self {
            base_evaluator,
            geometry,
            geometric_weight: geometric_weight.clamp(0.0, 1.0),
            target_concepts,
            avoid_concepts,
        }
    }

    pub async fn compute_reward(&self, reference: &str, kernel: &str) -> Result<f32, String> {
        // 1. Base (speedup + correção)
        let evaluation = self.base_evaluator.evaluate(reference, kernel).await?;
        if !evaluation.correct {
            return Ok(-2.0);
        }
        let speedup = evaluation.cuda_speedup_compile.max(0.1);
        let base_reward = (speedup.ln() / 8.0).clamp(-0.5, 2.5);

        // 2. Componente geométrica
        let kernel_emb = self.geometry.embed(kernel);
        let _reference_emb = self.geometry.embed(reference);

        // 2a. Alinhamento com conceitos alvo
        let mut target_score = 0.0;
        for concept in &self.target_concepts {
            if let Some(dir) = self.geometry.get_concept_direction(concept).await {
                target_score += self.geometry.causal_similarity(&kernel_emb.view(), &dir.view());
            }
        }
        if !self.target_concepts.is_empty() {
            target_score /= self.target_concepts.len() as f32;
        }

        // 2b. Ortogonalidade a conceitos indesejados
        let mut avoid_score = 0.0;
        for concept in &self.avoid_concepts {
            if let Some(dir) = self.geometry.get_concept_direction(concept).await {
                avoid_score += self.geometry.causal_orthogonality(&kernel_emb.view(), &dir.view());
            }
        }
        if !self.avoid_concepts.is_empty() {
            avoid_score /= self.avoid_concepts.len() as f32;
        }

        // 2c. Rank causal (quanto menor, mais abstrato/puro)
        let rank = self.geometry.causal_rank(&kernel_emb.view());
        let rank_score = 1.0 / (1.0 + rank as f32);

        let geometric_reward = (target_score + avoid_score + rank_score) / 3.0;

        // 3. Combinação
        let total =
            base_reward * (1.0 - self.geometric_weight) + geometric_reward * self.geometric_weight;

        debug!(
            "GeometricCudaReward: speedup={:.2}x, base={:.3}, geom={:.3}, total={:.3}",
            speedup, base_reward, geometric_reward, total
        );

        Ok(total)
    }
}
