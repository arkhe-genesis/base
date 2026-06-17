// Stub for EmbeddingModel to allow CausalGeometryService to compile
use ndarray::Array1;

pub trait EmbeddingModel {
    fn embed(&self, text: &str) -> Array1<f32>;
}

pub struct EmbeddingBridge;
