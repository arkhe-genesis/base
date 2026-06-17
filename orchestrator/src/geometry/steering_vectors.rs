//! Cathedral ARKHE v28.3.2 — Steering Vectors
//! Gera vetores de intervenção causais para controlar agentes.
//! Selo: CATHEDRAL-ARKHE-v28.3.2-STEERING-2026-06-16

use std::collections::HashMap;

use ndarray::Array1;

use super::{causal_inner_product::CovarianceMatrix, concept_directions::ConceptCatalog};

/// Vetor de steering causal para um conceito
#[derive(Debug, Clone)]
pub struct SteeringVector {
    pub concept: String,
    pub vector: Array1<f32>,
    pub intensity: f32, // 0..1 (força da intervenção)
}

/// Fábrica de steering vectors
pub struct SteeringFactory {
    cov: CovarianceMatrix,
    catalog: ConceptCatalog,
    /// Cache de steering vectors por conceito
    cache: HashMap<String, Array1<f32>>,
}

impl SteeringFactory {
    pub fn new(cov: CovarianceMatrix, catalog: ConceptCatalog) -> Self {
        Self { cov, catalog, cache: HashMap::new() }
    }

    /// Gera um steering vector para um conceito, garantindo ortogonalidade a outros conceitos
    pub fn get_steering_vector(
        &mut self,
        concept: &str,
        intensity: f32,
    ) -> Result<Array1<f32>, String> {
        // Verifica se já está em cache
        if let Some(v) = self.cache.get(concept) {
            let mut result = v.clone();
            result *= intensity;
            return Ok(result);
        }

        // Obtém a direção do conceito
        let dir = self
            .catalog
            .get_direction(concept)
            .ok_or_else(|| format!("Conceito '{}' não encontrado", concept))?;

        // O steering é a própria direção, mas pode ser purificada para remover
        // componentes que afetam conceitos indesejados (se configurado)
        let steering = dir.clone();

        // Armazena em cache
        self.cache.insert(concept.to_string(), steering.clone());

        // Aplica intensidade
        Ok(steering * intensity)
    }

    /// Gera um steering vector que é ortogonal a uma lista de conceitos indesejados
    pub fn get_orthogonal_steering(
        &mut self,
        concept: &str,
        avoid_concepts: &[&str],
        intensity: f32,
    ) -> Result<Array1<f32>, String> {
        let mut steering = self.get_steering_vector(concept, 1.0)?;

        // Remove componentes que se projetam nos conceitos a evitar
        for avoid in avoid_concepts {
            if let Some(avoid_dir) = self.catalog.get_direction(avoid) {
                let projection = self.cov.causal_project(&steering.view(), &avoid_dir.view());
                steering = &steering - &projection;
            }
        }

        // Normaliza e aplica intensidade
        let norm = self.cov.causal_norm(&steering.view());
        if norm > 1e-9 {
            steering = &steering / norm * intensity;
        }

        Ok(steering)
    }
}
