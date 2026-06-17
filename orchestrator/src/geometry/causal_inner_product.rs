//! Cathedral ARKHE v28.3.2 — Causal Inner Product (CIP)
//! Implementação do produto interno causal: ⟨γ, γ'⟩_C = γᵀ Cov(γ)⁻¹ γ'
//! Selo: CATHEDRAL-ARKHE-v28.3.2-CIP-2026-06-16

use nalgebra as na;
use ndarray::{Array1, Array2, ArrayView1};

/// Matriz de covariância dos unembedding vectors do modelo LLM
#[derive(Debug, Clone)]
pub struct CovarianceMatrix {
    /// Matriz de covariância (d x d)
    pub cov: Array2<f32>,
    /// Inversa da matriz de covariância (Cov⁻¹)
    pub cov_inv: Array2<f32>,
    /// Dimensão do espaço de embedding
    pub dimension: usize,
}

impl CovarianceMatrix {
    /// Estima a covariância a partir de um conjunto de vetores de entrada
    pub fn from_vectors(vectors: &[Array1<f32>]) -> Self {
        let n = vectors.len();
        if n == 0 {
            return Self::identity(vectors[0].len());
        }
        let d = vectors[0].len();

        // Calcula a média
        let mut mean = Array1::zeros(d);
        for v in vectors {
            mean += v;
        }
        mean /= n as f32;

        // Calcula a covariância amostral
        let mut cov = Array2::zeros((d, d));
        for v in vectors {
            let centered = v - &mean;
            for i in 0..d {
                for j in 0..d {
                    cov[[i, j]] += centered[i] * centered[j];
                }
            }
        }
        cov /= (n - 1) as f32;

        // Adiciona regularização (para evitar singularidade)
        let lambda = 1e-6;
        for i in 0..d {
            cov[[i, i]] += lambda;
        }

        // Calcula a inversa (usando nalgebra)
        let cov_na = na::DMatrix::from_vec(d, d, cov.iter().cloned().collect());
        let cov_inv_na = cov_na.clone().try_inverse().unwrap_or_else(|| {
            cov_na.pseudo_inverse(1e-6).unwrap_or_else(|_| na::DMatrix::zeros(d, d))
        });

        let cov_inv = Array2::from_shape_vec((d, d), cov_inv_na.as_slice().to_vec()).unwrap();

        Self { cov, cov_inv, dimension: d }
    }

    /// Matriz identidade (fallback)
    pub fn identity(d: usize) -> Self {
        let mut cov = Array2::zeros((d, d));
        let mut cov_inv = Array2::zeros((d, d));
        for i in 0..d {
            cov[[i, i]] = 1.0;
            cov_inv[[i, i]] = 1.0;
        }
        Self { cov, cov_inv, dimension: d }
    }

    /// Produto interno causal: ⟨a, b⟩_C = aᵀ Cov⁻¹ b
    pub fn causal_dot(&self, a: &ArrayView1<f32>, b: &ArrayView1<f32>) -> f32 {
        // aᵀ Cov⁻¹ b
        let temp = self.cov_inv.dot(b);
        a.dot(&temp)
    }

    /// Norma causal: ||v||_C = sqrt(vᵀ Cov⁻¹ v)
    pub fn causal_norm(&self, v: &ArrayView1<f32>) -> f32 {
        self.causal_dot(v, v).sqrt()
    }

    /// Similaridade causal: cos_c(a, b) = ⟨a, b⟩_C / (||a||_C * ||b||_C)
    pub fn causal_similarity(&self, a: &ArrayView1<f32>, b: &ArrayView1<f32>) -> f32 {
        let dot = self.causal_dot(a, b);
        let norm_a = self.causal_norm(a);
        let norm_b = self.causal_norm(b);
        if norm_a < 1e-9 || norm_b < 1e-9 {
            return 0.0;
        }
        dot / (norm_a * norm_b)
    }

    /// Projeção causal de v na direção de u: proj_u(v) = (⟨v, u⟩_C / ⟨u, u⟩_C) * u
    pub fn causal_project(&self, v: &ArrayView1<f32>, u: &ArrayView1<f32>) -> Array1<f32> {
        let dot_vu = self.causal_dot(v, u);
        let dot_uu = self.causal_dot(u, u);
        if dot_uu < 1e-9 {
            return Array1::zeros(v.len());
        }
        let coeff = dot_vu / dot_uu;
        u * coeff
    }

    /// Projeção ortogonal causal: v - proj_u(v)
    pub fn causal_orthogonalize(&self, v: &ArrayView1<f32>, u: &ArrayView1<f32>) -> Array1<f32> {
        let proj = self.causal_project(v, u);
        v - &proj
    }

    /// Mede a ortogonalidade causal entre dois vetores: 1 - |cos_c(a, b)|
    /// Valor próximo de 1 = ortogonais, próximo de 0 = alinhados
    pub fn causal_orthogonality(&self, a: &ArrayView1<f32>, b: &ArrayView1<f32>) -> f32 {
        1.0 - self.causal_similarity(a, b).abs()
    }
}
