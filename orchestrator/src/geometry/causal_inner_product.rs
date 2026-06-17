//! Cathedral ARKHE v28.3.2 — Causal Inner Product (CIP)
//! Núcleo matemático: ⟨γ, γ'⟩_C = γᵀ Cov⁻¹ γ'
//! Selo: CATHEDRAL-ARKHE-v28.3.2-CIP-2026-06-16

use ndarray::{Array1, Array2, ArrayView1};
use nalgebra as na;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CovarianceMatrix {
    pub cov: Array2<f32>,
    pub cov_inv: Array2<f32>,
    pub dimension: usize,
}

impl CovarianceMatrix {
    /// Estima a covariância a partir de um conjunto de vetores
    pub fn from_vectors(vectors: &[Array1<f32>]) -> Self {
        let n = vectors.len();
        if n == 0 {
            return Self::identity(vectors.first().map(|v| v.len()).unwrap_or(768));
        }
        let d = vectors[0].len();

        let mut mean = Array1::<f32>::zeros(d);
        for v in vectors { mean += v; }
        mean /= n as f32;

        let mut cov = Array2::<f32>::zeros((d, d));
        for v in vectors {
            let centered = v - &mean;
            cov += &centered.view().insert_axis(ndarray::Axis(1)).dot(&centered.view().insert_axis(ndarray::Axis(0)));
        }
        cov /= (n - 1) as f32;

        // Regularização (evita singularidade)
        let lambda = 1e-6;
        for i in 0..d { cov[[i, i]] += lambda; }

        let cov_na = na::DMatrix::from_vec(d, d, cov.iter().cloned().collect());
        let cov_inv_na = cov_na.clone().try_inverse()
            .unwrap_or_else(|| cov_na.pseudo_inverse(1e-6).unwrap());

        let cov_inv = Array2::from_shape_vec((d, d), cov_inv_na.as_slice().to_vec())
            .unwrap();

        Self { cov, cov_inv, dimension: d }
    }

    pub fn identity(d: usize) -> Self {
        let mut cov = Array2::<f32>::zeros((d, d));
        let mut cov_inv = Array2::<f32>::zeros((d, d));
        for i in 0..d {
            cov[[i, i]] = 1.0;
            cov_inv[[i, i]] = 1.0;
        }
        Self { cov, cov_inv, dimension: d }
    }

    /// Produto interno causal: ⟨a, b⟩_C = aᵀ Cov⁻¹ b
    pub fn causal_dot(&self, a: &ArrayView1<f32>, b: &ArrayView1<f32>) -> f32 {
        let temp = self.cov_inv.dot(b);
        a.dot(&temp)
    }

    /// Norma causal: ||v||_C = sqrt(vᵀ Cov⁻¹ v)
    pub fn causal_norm(&self, v: &ArrayView1<f32>) -> f32 {
        self.causal_dot(v, v).sqrt()
    }

    /// Similaridade causal (coseno causal)
    pub fn causal_similarity(&self, a: &ArrayView1<f32>, b: &ArrayView1<f32>) -> f32 {
        let dot = self.causal_dot(a, b);
        let na = self.causal_norm(a);
        let nb = self.causal_norm(b);
        if na < 1e-9 || nb < 1e-9 { return 0.0; }
        dot / (na * nb)
    }

    /// Ortogonalidade causal: 1 - |cos_c|
    pub fn causal_orthogonality(&self, a: &ArrayView1<f32>, b: &ArrayView1<f32>) -> f32 {
        1.0 - self.causal_similarity(a, b).abs()
    }

    /// Projeção causal de v na direção de u: proj_u(v) = (⟨v,u⟩_C / ⟨u,u⟩_C) * u
    pub fn causal_project(&self, v: &ArrayView1<f32>, u: &ArrayView1<f32>) -> Array1<f32> {
        let dot_vu = self.causal_dot(v, u);
        let dot_uu = self.causal_dot(u, u);
        if dot_uu < 1e-9 {
            return Array1::<f32>::zeros(v.len());
        }
        let coeff = dot_vu / dot_uu;
        let mut result = u.to_owned();
        result.map_mut(|x| *x *= coeff);
        result
    }

    /// Ortogonaliza v em relação a u: v - proj_u(v)
    pub fn causal_orthogonalize(&self, v: &ArrayView1<f32>, u: &ArrayView1<f32>) -> Array1<f32> {
        let p = self.causal_project(v, u);
        let mut result = v.to_owned();
        for i in 0..result.len() {
            result[i] -= p[i];
        }
        result
    }
}
