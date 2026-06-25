//! Implementação real do modelo usando llama-cpp-2.

use llama_cpp_2::{
    llama_backend::LlamaBackend,
    llama_batch::LlamaBatch,
    model::{params::LlamaModelParams, AddBos, LlamaModel},
    context::params::LlamaContextParams,
    sampling::LlamaSampler,
};
use std::num::NonZeroU32;
use std::path::PathBuf;
use std::sync::Arc;

/// Configuração do modelo.
#[derive(Debug, Clone)]
pub struct ModelConfig {
    pub model_path: PathBuf,
    pub n_ctx: u32,
    pub n_gpu_layers: u32,
}

impl Default for ModelConfig {
    fn default() -> Self {
        Self {
            model_path: PathBuf::from("models/agi.gguf"),
            n_ctx: 4096,
            n_gpu_layers: 35,
        }
    }
}

/// Motor de inferência real (não mock).
pub struct LlamaEngine {
    model: Arc<LlamaModel>,
    backend: Arc<LlamaBackend>,
    config: ModelConfig,
}

impl LlamaEngine {
    /// Carrega o modelo a partir do caminho configurado.
    pub fn load(config: ModelConfig) -> Result<Self, String> {
        let backend = LlamaBackend::init().map_err(|e| e.to_string())?;
        let backend = Arc::new(backend);

        let model_params = LlamaModelParams::default()
            .with_n_gpu_layers(config.n_gpu_layers);

        let model = LlamaModel::load_from_file(&backend, &config.model_path, &model_params)
            .map_err(|e| e.to_string())?;

        Ok(Self {
            model: Arc::new(model),
            backend,
            config,
        })
    }

    /// Gera uma resposta a partir de um prompt.
    pub fn generate(&self, prompt: &str, max_tokens: usize) -> Result<String, String> {
        let ctx_params = LlamaContextParams::default()
            .with_n_ctx(Some(NonZeroU32::new(self.config.n_ctx).unwrap_or(NonZeroU32::new(4096).unwrap())))
            .with_n_batch(512);

        let mut ctx = self.model
            .new_context(&self.backend, ctx_params)
            .map_err(|e| e.to_string())?;

        // Tokeniza o prompt
        let tokens = self.model
            .str_to_token(prompt, AddBos::Always)
            .map_err(|e| e.to_string())?;

        let n_tokens = tokens.len();
        let mut batch = LlamaBatch::new(n_tokens + max_tokens, 1);
        for (i, &tok) in tokens.iter().enumerate() {
            batch.add(tok, i as i32, &[0], i == tokens.len() - 1)
                .map_err(|e| e.to_string())?;
        }
        ctx.decode(&mut batch).map_err(|e| e.to_string())?;

        // ✅ CORREÇÃO: sampler com mut e sem penalties()
        let mut sampler = LlamaSampler::chain_simple(vec![
            LlamaSampler::temp(0.8),
            LlamaSampler::top_p(0.9, 1),
            LlamaSampler::top_k(40),
            LlamaSampler::greedy(),
        ]);

        let eos_token = self.model.token_eos();
        let mut generated = 0;
        let mut output_tokens = Vec::new();

        let mut decoder = encoding_rs::UTF_8.new_decoder();

        while generated < max_tokens {
            // ✅ CORREÇÃO: sample requer &mut self
            let token = sampler.sample(&ctx, -1);
            if token == eos_token {
                break;
            }

            // ✅ CORREÇÃO: token_to_piece com 4 parâmetros
            let piece = self.model
                .token_to_piece(token, &mut decoder, false, None)
                .map_err(|e| e.to_string())?;

            output_tokens.push(piece);
            generated += 1;

            let mut next_batch = LlamaBatch::new(1, 1);
            next_batch.add(token, 0, &[0], true)
                .map_err(|e| e.to_string())?;
            ctx.decode(&mut next_batch).map_err(|e| e.to_string())?;
        }

        Ok(output_tokens.concat())
    }
}
