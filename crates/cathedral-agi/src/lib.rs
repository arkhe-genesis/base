//! AGI Core v3.0.0 — com inferência real via Ollama e memória episódica
//! Selo: CATHEDRAL-ARKHE-AGI-CORE-v3.0.0-2026-06-19

mod agi_core;
mod ethics;
mod llm_client;
mod mcts;
mod meta_cognitive;
mod world_model;
mod wormhole;

pub use agi_core::AGICore;
pub use ethics::{EthicsResult, EthicsVerifier};
pub use llm_client::OllamaClient;
pub use mcts::{MCTSEngine, MCTSNode, MCTSResult};
pub use meta_cognitive::{MetaCognitiveLoop, MetaState};
pub use world_model::{Intent, WorldModel, WorldState};
pub use wormhole::HierarchicalWormhole;
