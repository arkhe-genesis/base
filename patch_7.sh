cat << 'INNER_EOF' > crates/safe-core-governance/src/mcp.rs
//! Servidor MCP para GovernanceEngine — Exposição dos 4 Pilares via MCP

use crate::governance::GovernanceEngine;
use rmcp::{
    ServerHandler, tool_router, tool_handler,
    ServiceExt, transport::stdio,
    model::{ServerInfo, ServerCapabilities},
};
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
        let service = self.serve(stdio()).await?;
        info!("Governance MCP Server iniciado via stdio");
        service.waiting().await?;
        Ok(())
    }
}

#[tool_handler]
impl ServerHandler for GovernanceMcpServer {
    fn get_info(&self) -> ServerInfo {
        ServerInfo {
            name: "Safe-Core Governance MCP Server".to_string(),
            version: env!("CARGO_PKG_VERSION").to_string(),
            instructions: Some(
                "Servidor de Governança para Agentes Autônomos.\n\n\
                 Os 4 Pilares:\n\
                 1. Ética: enforce_action — valida ações contra políticas\n\
                 2. Persistência: rule/* — CRUD de regras éticas\n\
                 3. Verificação: verify — verificação formal via Lean4\n\
                 4. Auditoria: audit/* — trilha imutável com Merkle"
                    .to_string()
            ),
            capabilities: ServerCapabilities::builder()
                .enable_tools()
                .build(),
        }
    }
}

#[tool_router]
impl GovernanceMcpServer {
    // ─── Pilar 1: Ética ───────────────────────────────────────────────────

    #[rmcp::tool(description = "Valida uma ação contra as políticas éticas (Pilar 1)")]
    async fn enforce_action(&self, action: String, context: serde_json::Value, agent_id: Option<String>) -> Result<serde_json::Value, String> {
        let verdict = self.engine
            .enforce_action(&action, &context, agent_id.as_deref())
            .await
            .map_err(|e| e.to_string())?;

        let output = serde_json::json!({
            "verdict": format!("{:?}", verdict),
            "reason": verdict.reason,
            "rule_id": verdict.rule_id,
        });

        Ok(serde_json::to_value(&output).unwrap_or_default())
    }

    // ─── Pilar 2: Persistência ────────────────────────────────────────────

    #[rmcp::tool(description = "Cria uma nova regra ética (Pilar 2)")]
    async fn create_rule(&self, action: String, constraint: String, severity: String, enabled: bool) -> Result<serde_json::Value, String> {
        use safe_core_ethics::{EthicsRule, Severity};

        let severity = match severity.as_str() {
            "block" => Severity::Block,
            "require_approval" => Severity::RequireApproval,
            _ => Severity::Allow,
        };

        let rule = EthicsRule {
            id: uuid::Uuid::new_v4().to_string(),
            action: action,
            constraint: constraint,
            severity,
            enabled: enabled,
        };

        self.engine.repository.save_rule(&rule).await
            .map_err(|e| e.to_string())?;

        Ok(serde_json::to_value(&rule).unwrap_or_default())
    }

    // ─── Pilar 3: Verificação ─────────────────────────────────────────────

    #[rmcp::tool(description = "Verifica uma restrição formalmente (Pilar 3)")]
    async fn verify_constraint(&self, constraint: String, context: serde_json::Value) -> Result<serde_json::Value, String> {
        use safe_core_verifier::Constraint;

        let constraint = Constraint {
            id: uuid::Uuid::new_v4().to_string(),
            expression: constraint,
            description: String::new(),
        };

        let result = self.engine.verify_constraint(&constraint, &context);

        let output = serde_json::json!({
            "valid": result.valid,
            "counterexample": result.counterexample,
        });

        Ok(serde_json::to_value(&output).unwrap_or_default())
    }

    // ─── Pilar 4: Auditoria ──────────────────────────────────────────────

    #[rmcp::tool(description = "Obtém a raiz da Merkle da trilha de auditoria (Pilar 4)")]
    async fn audit_root(&self) -> Result<serde_json::Value, String> {
        let root = self.engine.audit_root();

        let output = serde_json::json!({
            "root": root.map(|r| hex::encode(r)).unwrap_or_else(|| "N/A".to_string()),
        });

        Ok(serde_json::to_value(&output).unwrap_or_default())
    }

    #[rmcp::tool(description = "Lista os últimos eventos de auditoria")]
    async fn audit_events(&self) -> Result<serde_json::Value, String> {
        let audit = self.engine.audit.read().await;
        let events: Vec<_> = audit.events()
            .iter()
            .rev()
            .take(100)
            .map(|e| serde_json::json!({
                "id": e.id,
                "timestamp": e.timestamp.to_rfc3339(),
                "action": e.action,
                "verdict": e.verdict,
            }))
            .collect();

        Ok(serde_json::to_value(&events).unwrap_or_default())
    }
}
INNER_EOF
