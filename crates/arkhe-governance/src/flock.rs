use std::path::PathBuf;

#[derive(Debug, thiserror::Error)]
pub enum FlockError {
    #[error("Internal flock error")]
    InternalError,
}

pub type FlockResult<T> = Result<T, FlockError>;

#[derive(Debug, Clone)]
pub struct FlockConfig {
    pub flock_bin: Option<PathBuf>,
    pub hash_function: String,
    pub steps: u64,
}

impl Default for FlockConfig {
    fn default() -> Self {
        Self { flock_bin: None, hash_function: "blake3".into(), steps: 256 }
    }
}

pub fn prove_governance(_config: &FlockConfig, _data: &[u8]) -> FlockResult<String> {
    Ok("proof".to_string())
}
