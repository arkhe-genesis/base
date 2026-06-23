//! Expert individual do MoE

use crate::tensor::Tensor;
use crate::config::MoEConfig;

/// Expert individual — FFN com SwiGLU activation e clamping
pub struct Expert {
    /// Pesos da gate projection (hidden_size -> intermediate_size)
    pub gate_proj: Tensor,
    /// Pesos da up projection (hidden_size -> intermediate_size)
    pub up_proj: Tensor,
    /// Pesos da down projection (intermediate_size -> hidden_size)
    pub down_proj: Tensor,
    /// Bias da gate projection
    pub gate_bias: Tensor,
    /// Bias da up projection
    pub up_bias: Tensor,
    /// Bias da down projection
    pub down_bias: Tensor,
    /// Limite de clamping para SwiGLU
    pub clamp_limit: f32,
    /// Hidden size
    pub hidden_size: usize,
    /// Intermediate size
    pub intermediate_size: usize,
}

impl Expert {
    pub fn new(hidden_size: usize, intermediate_size: usize) -> Self {
        Self {
            gate_proj: Tensor::randn((hidden_size, intermediate_size)),
            up_proj: Tensor::randn((hidden_size, intermediate_size)),
            down_proj: Tensor::randn((intermediate_size, hidden_size)),
            gate_bias: Tensor::zeros((1, intermediate_size)),
            up_bias: Tensor::zeros((1, intermediate_size)),
            down_bias: Tensor::zeros((1, hidden_size)),
            clamp_limit: 10.0,
            hidden_size,
            intermediate_size,
        }
    }

    pub fn from_config(config: &MoEConfig) -> Self {
        Self::new(config.hidden_size, config.intermediate_size)
    }

    /// Forward pass do expert com SwiGLU Clamping
    pub fn forward(&self, x: &Tensor) -> Tensor {
        // 1. Gate projection
        let gate = x.matmul(&self.gate_proj).add(&self.gate_bias);

        // 2. Up projection
        let up = x.matmul(&self.up_proj).add(&self.up_bias);

        // 3. SwiGLU com clamping: gate * sigmoid(gate) * up
        let activated = self.swiglu_clamp(&gate, &up);

        // 4. Down projection
        activated.matmul(&self.down_proj).add(&self.down_bias)
    }

    /// SwiGLU activation com clamping para suprimir outliers
    fn swiglu_clamp(&self, gate: &Tensor, up: &Tensor) -> Tensor {
        // Clamp gate e up ao intervalo [-clamp_limit, clamp_limit]
        let g = gate.clamp(-self.clamp_limit, self.clamp_limit);
        let u = up.clamp(-self.clamp_limit, self.clamp_limit);

        // SwiGLU: gate * sigmoid(gate) * up
        let sig_g = g.sigmoid();
        g.mul_elem(&sig_g).mul_elem(&u)
    }

    /// Número de parâmetros do expert
    pub fn num_parameters(&self) -> usize {
        self.hidden_size * self.intermediate_size * 3
            + self.intermediate_size
            + self.intermediate_size
            + self.hidden_size
    }
}
