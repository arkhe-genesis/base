use thiserror::Error;

#[derive(Debug, Error, Clone, PartialEq)]
pub enum ParseError {
    #[error("Invalid format: {0}")]
    InvalidFormat(String),
    #[error("Invalid checksum: expected {expected}, got {actual}")]
    InvalidChecksum { expected: String, actual: String },
    #[error("Invalid length: expected {expected}, got {actual}")]
    InvalidLength { expected: usize, actual: usize },
    #[error("Suspicious characters detected: {0}")]
    SuspiciousCharacters(String),
    #[error("Regex timeout after {0:?}")]
    RegexTimeout(std::time::Duration),
    #[error("Input too large: {0} bytes (max {1})")]
    InputTooLarge(usize, usize),
    #[error("Dangerous regex pattern: {0}")]
    DangerousPattern(String),
    #[error("Regex compilation failed: {0}")]
    RegexCompilationFailed(String),
    #[error("Nom parse error: {0}")]
    NomError(String),
    #[error("Unknown parse error: {0}")]
    Unknown(String),
}

pub type ParseResult<T> = Result<T, ParseError>;
