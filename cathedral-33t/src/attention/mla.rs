//! Multi-Head Latent Attention (MLA)

use crate::tensor::Tensor;
use crate::config::AttentionConfig;

pub struct MultiHeadLatentAttention {
    pub latent_dim: usize,
    pub head_dim: usize,
    pub num_heads: usize,
    pub kv_compression: Tensor,
    pub kv_decompression: Tensor,
}

impl MultiHeadLatentAttention {
    pub fn new(config: &AttentionConfig) -> Self {
        let d_model = config.num_heads * config.head_dim;
        Self {
            latent_dim: config.mla_latent_dim,
            head_dim: config.head_dim,
            num_heads: config.num_heads,
            kv_compression: Tensor::randn((d_model, config.mla_latent_dim)),
            kv_decompression: Tensor::randn((config.mla_latent_dim, d_model)),
        }
    }

    pub fn forward(&self, x: &Tensor, kv_cache: Option<&Tensor>) -> (Tensor, Tensor) {
        let compressed = x.matmul(&self.kv_compression);

        let new_kv = if let Some(_cache) = kv_cache {
            compressed.clone()
        } else {
            compressed.clone()
        };

        let decompressed = new_kv.matmul(&self.kv_decompression);
        (decompressed, new_kv)
    }

    pub fn compression_ratio(&self) -> f32 {
        (self.num_heads * self.head_dim) as f32 / self.latent_dim as f32
    }
}
