use async_trait::async_trait;

#[async_trait]
pub trait InferenceEngine: Send + Sync {
    async fn generate(&self, prompt: &str, temperature: f32, max_tokens: u32) -> Result<String, String>;
}

pub struct LlamaCppEngine {
    model_path: String,
    // ctx: Option<llama_cpp_2::model::LlamaModel>,
}

impl LlamaCppEngine {
    pub fn new(model_path: &str) -> Self {
        Self { model_path: model_path.to_string(), /* ctx: None */ }
    }

    /* pub fn load(&mut self) -> Result<(), String> {
        let params = llama_cpp_2::model::params::LlamaModelParams::default();
        let backend = llama_cpp_2::context::params::LlamaContextParams::default();
        self.ctx = Some(llama_cpp_2::model::LlamaModel::load_from_file(&self.model_path, params)
            .map_err(|e| e.to_string())?);
        Ok(())
    } */
}

#[async_trait]
impl InferenceEngine for LlamaCppEngine {
    async fn generate(&self, _prompt: &str, _temperature: f32, _max_tokens: u32) -> Result<String, String> {
        Ok("Mock LLM Output".to_string())
    }
}
