//! Cathedral ARKHE v28.3.2 — Concept Directions
//! Extrai e mantém direções causais para conceitos conhecidos.
//! Selo: CATHEDRAL-ARKHE-v28.3.2-CONCEPT-DIRS-2026-06-16

use ndarray::Array1;
use std::collections::HashMap;
use std::sync::Arc;
use tracing::debug;

use super::causal_inner_product::CovarianceMatrix;

/// Direção de um conceito no espaço causal
#[derive(Debug, Clone)]
pub struct ConceptDirection {
    pub name: String,
    pub vector: Array1<f32>,
    pub confidence: f32,     // 0..1 (quão bem definido)
    pub sample_count: usize, // número de exemplos usados
}

/// Catálogo de conceitos conhecidos
#[derive(Clone)]
pub struct ConceptCatalog {
    concepts: HashMap<String, ConceptDirection>,
    cov: Arc<CovarianceMatrix>,
}

impl ConceptCatalog {
    pub fn new(cov: Arc<CovarianceMatrix>) -> Self {
        Self { concepts: HashMap::new(), cov }
    }

    /// Registra um conceito a partir de exemplos positivos e negativos
    pub fn register_concept(
        &mut self,
        name: &str,
        positive_examples: &[Array1<f32>],
        negative_examples: &[Array1<f32>],
    ) -> Result<(), String> {
        if positive_examples.is_empty() || negative_examples.is_empty() {
            return Err("Exemplos insuficientes".into());
        }

        // Calcula a média causal dos exemplos positivos
        let mut pos_mean = Array1::zeros(self.cov.dimension);
        for ex in positive_examples {
            pos_mean += ex;
        }
        pos_mean /= positive_examples.len() as f32;

        // Calcula a média causal dos exemplos negativos
        let mut neg_mean = Array1::zeros(self.cov.dimension);
        for ex in negative_examples {
            neg_mean += ex;
        }
        neg_mean /= negative_examples.len() as f32;

        // A direção do conceito é a diferença entre as médias
        let direction = pos_mean - &neg_mean;

        // Normaliza causalmente
        let norm = self.cov.causal_norm(&direction.view());
        if norm < 1e-9 {
            return Err("Direção do conceito tem norma zero".into());
        }
        let normalized = direction / norm;

        self.concepts.insert(
            name.to_string(),
            ConceptDirection {
                name: name.to_string(),
                vector: normalized,
                confidence: 1.0, // placeholder: calcularia com validação cruzada
                sample_count: positive_examples.len(),
            },
        );

        debug!("Conceito '{}' registrado com {} amostras", name, positive_examples.len());
        Ok(())
    }

    /// Obtém a direção de um conceito
    pub fn get_direction(&self, name: &str) -> Option<Array1<f32>> {
        self.concepts.get(name).map(|c| c.vector.clone())
    }

    /// Mede a ortogonalidade entre dois conceitos
    pub fn orthogonality(&self, concept_a: &str, concept_b: &str) -> Option<f32> {
        let dir_a = self.get_direction(concept_a)?;
        let dir_b = self.get_direction(concept_b)?;
        Some(self.cov.causal_orthogonality(&dir_a.view(), &dir_b.view()))
    }

    /// Retorna conceitos que são ortogonais a um dado conceito (threshold > 0.7)
    pub fn get_orthogonal_concepts(&self, concept: &str, threshold: f32) -> Vec<String> {
        let dir = match self.get_direction(concept) {
            Some(d) => d,
            None => return vec![],
        };
        self.concepts
            .iter()
            .filter(|(name, _)| *name != concept)
            .filter_map(|(name, c)| {
                let orth = self.cov.causal_orthogonality(&dir.view(), &c.vector.view());
                if orth > threshold { Some(name.clone()) } else { None }
            })
            .collect()
    }
}
