//! Cathedral ARKHE v14.1 — Cognitive Singularity Engine (CSE)
//!
//! Esta crate integra todos os componentes da v14.1:
//! - MoE Cognitive Orchestrator
//! - Thinking Engine (Chain-of-Thought)
//! - Spatial Attention Engine (MSA-style)
//! - Multi-Token Prediction (MTP)
//! - SAHOO+ (Alinhamento Adaptativo)
//! - Cathedral Code Agent 2.0 (CCA 2.0)
//!
//! Reutiliza os substratos da v13.1 (Trinity, EAC, Mesh, Tools).

pub mod agent;
pub mod attention;
pub mod moe;
pub mod mtp;
pub mod sahoo;
pub mod thinking;
pub mod tools;
pub mod trinity;

// Reexportações para facilitar o uso
pub use agent::*;
pub use attention::*;
pub use moe::*;
pub use mtp::*;
pub use sahoo::*;
pub use thinking::*;
