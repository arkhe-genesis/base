#[derive(Debug, thiserror::Error)]
pub enum FlockError {
    #[error("Internal flock error")]
    InternalError,
}

pub type FlockResult<T> = Result<T, FlockError>;
