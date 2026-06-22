//! Hybrid Attention: CSA + HCA + MLA + Sliding Window

mod csa;
mod hca;
mod sliding_window;
mod mla;
mod rope;

pub use csa::CompressedSparseAttention;
pub use hca::HeavilyCompressedAttention;
pub use sliding_window::SlidingWindowAttention;
pub use mla::MultiHeadLatentAttention;
pub use rope::{compute_rope_frequencies, apply_rope};

use crate::tensor::Tensor;
use crate::config::AttentionConfig;

pub struct HybridAttention {
    pub csa: CompressedSparseAttention,
    pub hca: HeavilyCompressedAttention,
    pub sliding: SlidingWindowAttention,
    pub mla: MultiHeadLatentAttention,
    csa_weight: Tensor,
    hca_weight: Tensor,
    sliding_weight: Tensor,
}

impl HybridAttention {
    pub fn new(config: &AttentionConfig) -> Self {
        Self {
            csa: CompressedSparseAttention::new(config),
            hca: HeavilyCompressedAttention::new(config),
            sliding: SlidingWindowAttention::new(config),
            mla: MultiHeadLatentAttention::new(config),
            csa_weight: Tensor::randn((1, 1)),
            hca_weight: Tensor::randn((1, 1)),
            sliding_weight: Tensor::randn((1, 1)),
        }
    }

    pub fn forward(&self, x: &Tensor, kv_cache: Option<&Tensor>) -> (Tensor, Tensor) {
        let csa_out = self.csa.forward(x, kv_cache);
        let hca_out = self.hca.forward(x, kv_cache);
        let slide_out = self.sliding.forward(x, kv_cache);

        let combined = self.combine(csa_out, hca_out, slide_out);
        self.mla.forward(&combined, kv_cache)
    }

    fn combine(&self, a: Tensor, b: Tensor, c: Tensor) -> Tensor {
        let w_csa = self.csa_weight.data[[0, 0]].max(0.0);
        let w_hca = self.hca_weight.data[[0, 0]].max(0.0);
        let w_slide = self.sliding_weight.data[[0, 0]].max(0.0);

        let total = w_csa + w_hca + w_slide;
        let norm = if total > 0.0 { 1.0 / total } else { 1.0 };

        a.scale(w_csa * norm)
            .add(&b.scale(w_hca * norm))
            .add(&c.scale(w_slide * norm))
    }
}
