//! Rotary Position Embedding (RoPE)

use crate::tensor::Tensor;

pub fn compute_rope_frequencies(dim: usize, theta: f32, max_seq_len: usize) -> (Vec<f32>, Vec<f32>) {
    let inv_freq: Vec<f32> = (0..dim / 2)
        .map(|i| 1.0 / theta.powf(2.0 * i as f32 / dim as f32))
        .collect();

    let mut cos_cache = Vec::with_capacity(max_seq_len * dim / 2);
    let mut sin_cache = Vec::with_capacity(max_seq_len * dim / 2);

    for pos in 0..max_seq_len {
        for freq in &inv_freq {
            let angle = pos as f32 * freq;
            cos_cache.push(angle.cos());
            sin_cache.push(angle.sin());
        }
    }

    (cos_cache, sin_cache)
}

pub fn apply_rope(x: &Tensor, cos_cache: &[f32], sin_cache: &[f32], pos: usize, head_dim: usize) -> Tensor {
    let mut result = x.clone();
    let half_dim = head_dim / 2;

    // Simplificado. Na implementação real é necessário usar mapv ou iteração no tensor
    // para cada token

    for i in 0..half_dim {
        let cos = cos_cache[pos * half_dim + i];
        let sin = sin_cache[pos * half_dim + i];

        let x0 = result.data[[0, i]];
        let x1 = result.data[[0, i + half_dim]];

        result.data[[0, i]] = x0 * cos - x1 * sin;
        result.data[[0, i + half_dim]] = x0 * sin + x1 * cos;
    }

    result
}
