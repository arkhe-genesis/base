//! Cathedral ARKHE v28.3.2 — Geometric Policy Engine
//! Políticas baseadas em ortogonalidade e subespaços causais.
//! Selo: CATHEDRAL-ARKHE-v28.3.2-GEOMETRIC-POLICY-2026-06-16

use ndarray::{Array1, ArrayView1};
use std::collections::HashMap;
use std::sync::Arc;

use crate::geometry::service::CausalGeometryService;
use crate::orchestrator::AgentRole;

#[derive(Debug, Clone)]
pub enum GeometricPolicyType {
    /// A saída deve ser ortogonal a conceitos proibidos
    OutputOrthogonalTo { forbidden: Vec<String>, threshold: f32 },
    /// Steering não pode afetar conceitos off-target
    SteeringTargeted { target: String, max_correlation: f32 },
    /// Representação não pode colapsar conceitos
    NoRepresentationCollapse { min_orthogonality: f32 },
    /// Conceitos devem ser separáveis
    ConceptsMustBeSeparable { a: String, b: String, min_separation: f32 },
}

#[derive(Debug, Clone)]
pub struct GeometricPolicy {
    pub name: String,
    pub enabled: bool,
    pub policy_type: GeometricPolicyType,
}

pub struct GeometricPolicyEngine {
    policies: Vec<GeometricPolicy>,
    geometry: Arc<CausalGeometryService>,
    /// Subespaços proibidos (cache de direções)
    _forbidden_subspaces: HashMap<String, Vec<Array1<f32>>>,
}

impl GeometricPolicyEngine {
    pub fn new(geometry: Arc<CausalGeometryService>) -> Self {
        let mut engine =
            Self { policies: Vec::new(), geometry, _forbidden_subspaces: HashMap::new() };
        engine.register_default_policies();
        engine
    }

    fn register_default_policies(&mut self) {
        // Política 1: PII proibida na saída
        self.policies.push(GeometricPolicy {
            name: "pii_prohibition".to_string(),
            enabled: true,
            policy_type: GeometricPolicyType::OutputOrthogonalTo {
                forbidden: vec!["person".into(), "email".into(), "phone".into()],
                threshold: 0.7,
            },
        });

        // Política 2: Steering não pode afetar segurança
        self.policies.push(GeometricPolicy {
            name: "steering_safety".to_string(),
            enabled: true,
            policy_type: GeometricPolicyType::SteeringTargeted {
                target: "safety".into(),
                max_correlation: 0.3,
            },
        });

        // Política 3: Representação saudável (sem colapso)
        self.policies.push(GeometricPolicy {
            name: "representation_health".to_string(),
            enabled: true,
            policy_type: GeometricPolicyType::NoRepresentationCollapse { min_orthogonality: 0.25 },
        });

        // Política 4: Separação entre conceitos
        self.policies.push(GeometricPolicy {
            name: "concept_separation".to_string(),
            enabled: true,
            policy_type: GeometricPolicyType::ConceptsMustBeSeparable {
                a: "code".into(),
                b: "safety".into(),
                min_separation: 0.4,
            },
        });
    }

    /// Autoriza uma ação/saída com base na geometria causal
    pub async fn authorize(
        &self,
        _role: AgentRole,
        action: &str,
        output: &str,
        action_emb: Option<&Array1<f32>>,
        output_emb: Option<&Array1<f32>>,
    ) -> Result<(), String> {
        let action_emb_generated;
        let action_vec = match action_emb {
            Some(e) => e.view(),
            None => {
                action_emb_generated = self.geometry.embed(action);
                action_emb_generated.view()
            }
        };

        let output_emb_generated;
        let output_vec = match output_emb {
            Some(e) => e.view(),
            None => {
                output_emb_generated = self.geometry.embed(output);
                output_emb_generated.view()
            }
        };

        for policy in &self.policies {
            if !policy.enabled {
                continue;
            }

            match &policy.policy_type {
                GeometricPolicyType::OutputOrthogonalTo { forbidden, threshold } => {
                    for concept in forbidden {
                        if let Some(dir) = self.geometry.get_concept_direction(concept).await {
                            let orth = self.geometry.causal_orthogonality(&output_vec, &dir.view());
                            if orth < *threshold {
                                return Err(format!(
                                    "Violação da política '{}': saída não é ortogonal a '{}' (orth={:.3} < {})",
                                    policy.name, concept, orth, threshold
                                ));
                            }
                        }
                    }
                }
                GeometricPolicyType::SteeringTargeted { target, max_correlation } => {
                    if let Some(dir) = self.geometry.get_concept_direction(target).await {
                        let sim = self.geometry.causal_similarity(&action_vec, &dir.view());
                        if sim > *max_correlation {
                            return Err(format!(
                                "Violação da política '{}': steering correlacionado com '{}' (sim={:.3} > {})",
                                policy.name, target, sim, max_correlation
                            ));
                        }
                    }
                }
                GeometricPolicyType::NoRepresentationCollapse { min_orthogonality } => {
                    let diversity = self.measure_representation_diversity(&output_vec).await;
                    if diversity < *min_orthogonality {
                        return Err(format!(
                            "Violação da política '{}': representação colapsada (diversity={:.3} < {})",
                            policy.name, diversity, min_orthogonality
                        ));
                    }
                }
                GeometricPolicyType::ConceptsMustBeSeparable { a, b, min_separation } => {
                    let orth = self.geometry.concept_orthogonality(a, b).await.unwrap_or(0.0);
                    if orth < *min_separation {
                        return Err(format!(
                            "Violação da política '{}': conceitos '{}' e '{}' não são separáveis (orth={:.3} < {})",
                            policy.name, a, b, orth, min_separation
                        ));
                    }
                }
            }
        }

        Ok(())
    }

    async fn measure_representation_diversity(&self, emb: &ArrayView1<'_, f32>) -> f32 {
        let concepts = vec!["person", "email", "code", "safety", "efficiency", "memory"];
        let mut projections = Vec::new();
        for concept in concepts {
            if let Some(dir) = self.geometry.get_concept_direction(concept).await {
                projections.push(self.geometry.causal_dot(emb, &dir.view()));
            }
        }
        if projections.len() < 2 {
            return 0.5;
        }
        let mean = projections.iter().sum::<f32>() / projections.len() as f32;
        let variance =
            projections.iter().map(|x| (x - mean).powi(2)).sum::<f32>() / projections.len() as f32;
        (variance / 0.5).clamp(0.0, 1.0)
    }
}
