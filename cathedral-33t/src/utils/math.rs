//! Funções matemáticas utilitárias

use crate::tensor::Tensor;

/// Sigmoid
pub fn sigmoid(x: &Tensor) -> Tensor {
    x.sigmoid()
}

/// RMS Normalization
pub fn rms_norm(x: &Tensor, eps: f32) -> Tensor {
    let mean_sq = x.mapv(|v| v * v).mean_axis(1);
    let scale = mean_sq.sqrt().mapv(move |v| 1.0 / (v + eps));
    x.mul_elem(&scale)
}

/// Layer Normalization
pub fn layer_norm(x: &Tensor, eps: f32) -> Tensor {
    let mean = x.mean_axis(1);
    let var = x.sub(&mean).mapv(|v| v*v).mean_axis(1);
    let std = var.mapv(move |v| (v + eps).sqrt());
    x.sub(&mean).mul_elem(&std.mapv(|v| 1.0 / v))
}

/// Softmax ao longo do eixo especificado
pub fn softmax(x: &Tensor, axis: usize) -> Tensor {
    let max = x.max();
    let exp = x.mapv(|v| (v - max).exp());
    let sum = exp.sum_axis(axis);
    exp.mul_elem(&sum.mapv(|v| 1.0 / v))
}

/// GELU activation
pub fn gelu(x: &Tensor) -> Tensor {
    x.mapv(|v| {
        let cdf = 0.5 * (1.0 + (v * 0.7978845608 * (1.0 + 0.044715 * v * v)).tanh());
        v * cdf
    })
}

/// ReLU activation
pub fn relu(x: &Tensor) -> Tensor {
    x.mapv(|v| v.max(0.0))
}

/// SwiGLU com clamping
pub fn swiglu_clamp(gate: &Tensor, up: &Tensor, clamp_limit: f32) -> Tensor {
    let g = gate.clamp(-clamp_limit, clamp_limit);
    let u = up.clamp(-clamp_limit, clamp_limit);
    let sig_g = g.sigmoid();
    g.mul_elem(&sig_g).mul_elem(&u)
}

/// Gradient clipping
pub fn clip_gradients(grads: &mut [Tensor], max_norm: f32) {
    let mut total_norm = 0.0f32;
    for grad in grads.iter() {
        total_norm += grad.mapv(|v| v * v).sum();
    }
    total_norm = total_norm.sqrt();

    if total_norm > max_norm {
        let scale = max_norm / total_norm;
        for grad in grads.iter_mut() {
            *grad = grad.scale(scale);
        }
    }
}

/// Cosine learning rate schedule
pub fn cosine_lr_schedule(
    step: u64,
    warmup_steps: u64,
    total_steps: u64,
    max_lr: f64,
    min_lr: f64,
) -> f64 {
    if step < warmup_steps {
        max_lr * (step as f64 / warmup_steps as f64)
    } else {
        let progress = (step - warmup_steps) as f64 / (total_steps - warmup_steps) as f64;
        min_lr + (max_lr - min_lr) * 0.5 * (1.0 + (std::f64::consts::PI * progress).cos())
    }
}
