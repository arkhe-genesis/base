//! TEEBridge — Atestação unificada SGX + IoNet
//! Selo: CATHEDRAL-ARKHE-TEE-v1.0.0-2026-06-19

mod bridge;
pub mod secure_vm;
mod types;
mod verifier;

pub use bridge::TEEBridge;
pub use secure_vm::SecureVmExecutor;
pub use types::{AttestationReport, AttestationResult, TeeType};
