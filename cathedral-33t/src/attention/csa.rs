//! Compressed Sparse Attention (CSA)

use crate::tensor::Tensor;
use crate::config::AttentionConfig;

pub struct CompressedSparseAttention {
    num_heads: usize,
    head_dim: usize,
    compression_ratio: usize,
    q_proj: Tensor,
    k_proj: Tensor,
    v_proj: Tensor,
    o_proj: Tensor,
}

impl CompressedSparseAttention {
    pub fn new(config: &AttentionConfig) -> Self {
        let head_dim = config.head_dim;
        let num_heads = config.num_heads;
        let d_model = num_heads * head_dim;
        let compressed_dim = d_model / config.csa_compression;

        Self {
            num_heads,
            head_dim,
            compression_ratio: config.csa_compression,
            q_proj: Tensor::randn((d_model, d_model)),
            k_proj: Tensor::randn((d_model, compressed_dim)),
            v_proj: Tensor::randn((d_model, compressed_dim)),
            o_proj: Tensor::randn((d_model, d_model)),
        }
    }

    pub fn forward(&self, x: &Tensor, _kv_cache: Option<&Tensor>) -> Tensor {
        let q = x.matmul(&self.q_proj);
        let k = x.matmul(&self.k_proj);
        let v = x.matmul(&self.v_proj);

        let scores = q.matmul(&k.transpose());
        // Need to add softmax axis parameter here appropriately based on ndarray implementation
        // Defaulting to axis=1 for illustration
        let attn = crate::utils::math::softmax(&scores, 1);
        let out = attn.matmul(&v);

        out.matmul(&self.o_proj)
    }
}
