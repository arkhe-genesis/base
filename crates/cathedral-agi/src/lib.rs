//! AGI Core v3.0.0 — com inferência real via Ollama e memória episódica
//! Selo: CATHEDRAL-ARKHE-AGI-CORE-v3.0.0-2026-06-19

mod world_model;
mod mcts;
mod meta_cognitive;
mod wormhole;
mod ethics;
mod agi_core;
mod llm_client;

pub use agi_core::AGICore;
pub use world_model::{WorldModel, WorldState, Intent};
pub use mcts::{MCTSEngine, MCTSNode, MCTSResult};
pub use meta_cognitive::{MetaCognitiveLoop, MetaState};
pub use wormhole::HierarchicalWormhole;
pub use ethics::{EthicsVerifier, EthicsResult};
pub use llm_client::OllamaClient;
