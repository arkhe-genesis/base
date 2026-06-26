use thiserror::Error;

#[derive(Error, Debug, Clone, PartialEq)]
pub enum KernelError {
    #[error("Generic error")]
    Generic,
    #[error("Invalid capability")]
    InvalidCapability,
    #[error("Capability expired")]
    CapabilityExpired,
    #[error("Agent not found: {0}")]
    AgentNotFound(String),
    #[error("Out of memory")]
    OutOfMemory,
    #[error("IPC error: {0}")]
    IpcError(String),
    #[error("Invalid syscall: {0}")]
    InvalidSyscall(u32),
    #[error("Invalid proof")]
    InvalidProof,
    #[error("PQC error: {0}")]
    PqcError(String),
    #[error("Adapter not found")]
    AdapterNotFound,
    #[error("Unsupported asset: {0}")]
    UnsupportedAsset(String),
    #[error("Timeout")]
    Timeout,
    #[error("Insufficient balance")]
    InsufficientBalance,
    #[error("Serialization error: {0}")]
    Serialization(String),
}

pub type Result<T> = std::result::Result<T, KernelError>;
