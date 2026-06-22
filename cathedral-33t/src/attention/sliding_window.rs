//! Sliding Window Attention

use crate::tensor::Tensor;
use crate::config::AttentionConfig;

pub struct SlidingWindowAttention {
    pub window_size: usize,
}

impl SlidingWindowAttention {
    pub fn new(config: &AttentionConfig) -> Self {
        Self {
            window_size: config.sliding_window_size,
        }
    }

    pub fn forward(&self, x: &Tensor, _kv_cache: Option<&Tensor>) -> Tensor {
        // Simplified
        x.clone()
    }
}
