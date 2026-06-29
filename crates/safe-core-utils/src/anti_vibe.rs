//! 🚨 Anti-Vibe Attractor — Catálogo Formal de Falhas do Vibe-Coding
//!
//! Este módulo documenta casos reais de falhas em código gerado por IA
//! e os utiliza como "attractors" negativos no Coherence-Gradient Following.
//!
//! # Convenção X
//! - `x_detect_vibe_awareness` — analisa respostas de fronteira (LLM)
//! - `VibeFailScenario` — estrutura de dados imutável (core)
//! - `VIBE_FAILS` — catálogo público de cenários documentados

use serde::{Deserialize, Serialize};

/// Cenário documentado de falha no vibe-coding.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VibeFailScenario {
    pub title: String,
    pub industry: String,
    pub time_lost_hours: u32,
    pub monetary_cost: Option<String>,
    pub bad_code_pattern: String,
    pub safe_core_solution: String,
    pub source: String,
    pub severity: u8,
}
