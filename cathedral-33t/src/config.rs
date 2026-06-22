//! Estruturas de configuração para Cathedral 33T

use serde::{Deserialize, Serialize};

/// Configuração raiz do modelo Cathedral 33T
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CathedralConfig {
    pub model: ModelConfig,
    pub training: TrainingConfig,
    pub inference: InferenceConfig,
}

/// Configuração do modelo
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelConfig {
    pub hidden_size: usize,
    pub num_layers: usize,
    pub vocab_size: usize,
    pub max_seq_len: usize,
    pub num_experts: usize,
    pub top_k: usize,
    pub intermediate_size: usize,
    pub mhc_expansion_rate: usize,
    pub capacity_factor: f32,
    pub load_balancing_loss_coef: f32,
    pub moe: MoEConfig,
    pub attention: AttentionConfig,
    pub quantization: QuantizationConfig,
}

/// Configuração do MoE
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MoEConfig {
    pub num_experts: usize,
    pub top_k: usize,
    pub hidden_size: usize,
    pub intermediate_size: usize,
    pub capacity_factor: f32,
    pub load_balancing_loss_coef: f32,
}

/// Configuração da atenção
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AttentionConfig {
    pub num_heads: usize,
    pub head_dim: usize,
    pub num_kv_heads: usize,
    pub csa_compression: usize,
    pub hca_compression: usize,
    pub sliding_window_size: usize,
    pub mla_latent_dim: usize,
    pub rope_theta: f32,
}

/// Configuração de quantização
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QuantizationConfig {
    pub weight_precision: String,
    pub activation_precision: String,
    pub router_precision: String,
    pub use_mxfp4: bool,
    pub use_fp8: bool,
}

/// Configuração de treino
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrainingConfig {
    pub total_steps: u64,
    pub batch_size: usize,
    pub learning_rate: f64,
    pub optimizer: String,
    pub beta_a: f32,
    pub alpha: f32,
    pub warmup_steps: u64,
    pub gradient_clip_norm: f64,
    pub checkpoint_interval: u64,
    pub eval_interval: u64,
    pub fp4_quantization: bool,
    pub fp8_mixed_precision: bool,
}

/// Configuração de inferência
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InferenceConfig {
    pub speculative_tokens: usize,
    pub draft_model_path: String,
    pub batch_size: usize,
    pub quantize: bool,
    pub max_context_length: usize,
    pub temperature: f32,
    pub top_p: f32,
}

impl Default for CathedralConfig {
    fn default() -> Self {
        Self {
            model: ModelConfig::default(),
            training: TrainingConfig::default(),
            inference: InferenceConfig::default(),
        }
    }
}

impl Default for ModelConfig {
    fn default() -> Self {
        Self {
            hidden_size: 8192,
            num_layers: 128,
            vocab_size: 100_000,
            max_seq_len: 1_000_000,
            num_experts: 4096,
            top_k: 8,
            intermediate_size: 32768,
            mhc_expansion_rate: 4,
            capacity_factor: 1.25,
            load_balancing_loss_coef: 0.01,
            moe: MoEConfig::default(),
            attention: AttentionConfig::default(),
            quantization: QuantizationConfig::default(),
        }
    }
}

impl Default for MoEConfig {
    fn default() -> Self {
        Self {
            num_experts: 4096,
            top_k: 8,
            hidden_size: 8192,
            intermediate_size: 32768,
            capacity_factor: 1.25,
            load_balancing_loss_coef: 0.01,
        }
    }
}

impl Default for AttentionConfig {
    fn default() -> Self {
        Self {
            num_heads: 64,
            head_dim: 128,
            num_kv_heads: 8,
            csa_compression: 4,
            hca_compression: 128,
            sliding_window_size: 4096,
            mla_latent_dim: 512,
            rope_theta: 10000.0,
        }
    }
}

impl Default for QuantizationConfig {
    fn default() -> Self {
        Self {
            weight_precision: "FP4".to_string(),
            activation_precision: "FP8".to_string(),
            router_precision: "FP32".to_string(),
            use_mxfp4: true,
            use_fp8: true,
        }
    }
}

impl Default for TrainingConfig {
    fn default() -> Self {
        Self {
            total_steps: 1_000_000,
            batch_size: 1024,
            learning_rate: 1e-4,
            optimizer: "MONA".to_string(),
            beta_a: 0.975,
            alpha: -20.0,
            warmup_steps: 10000,
            gradient_clip_norm: 1.0,
            checkpoint_interval: 1000,
            eval_interval: 500,
            fp4_quantization: true,
            fp8_mixed_precision: true,
        }
    }
}

impl Default for InferenceConfig {
    fn default() -> Self {
        Self {
            speculative_tokens: 5,
            draft_model_path: "/models/draft.bin".to_string(),
            batch_size: 64,
            quantize: true,
            max_context_length: 1_000_000,
            temperature: 0.7,
            top_p: 0.9,
        }
    }
}
