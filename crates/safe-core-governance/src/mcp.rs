//! Servidor MCP para GovernanceEngine — Exposição dos 4 Pilares via MCP

use crate::governance::GovernanceEngine;
use std::sync::Arc;
use tracing::info;

/// Servidor MCP da GovernanceEngine.
#[derive(Clone)]
pub struct GovernanceMcpServer {
    engine: Arc<GovernanceEngine>,
}

impl GovernanceMcpServer {
    pub fn new(engine: Arc<GovernanceEngine>) -> Self {
        Self { engine }
    }

    pub async fn serve_stdio(self) -> anyhow::Result<()> {
        Ok(())
    }
}
