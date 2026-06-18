//! Cathedral ARKHE v28.3.2 — MCP Subagent Handler
//! Ferramentas MCP para gerenciar subagentes via API.
//! Selo: CATHEDRAL-ARKHE-v28.3.2-MCP-SUBAGENT-2026-06-17

use std::sync::Arc;
use serde_json::json;
use tracing::{info, warn};

use crate::subagent_spawner::SubagentSpawner;
use crate::mcp::server::{MCPError, MCPServerState};

// ============================================================================
// 1. Handler: spawn_subagent
// ============================================================================

pub async fn handle_spawn_subagent(
    args: serde_json::Value,
    state: &Arc<MCPServerState>,
) -> Result<serde_json::Value, MCPError> {
    let spawner = state.subagent_spawner.as_ref()
        .ok_or_else(|| MCPError {
            code: -32000,
            message: "SubagentSpawner não disponível".to_string(),
            data: None,
        })?;

    let purpose = args
        .get("purpose")
        .and_then(|v| v.as_str())
        .ok_or_else(|| MCPError {
            code: -32602,
            message: "Campo 'purpose' obrigatório".to_string(),
            data: None,
        })?;

    let tools: Vec<String> = args
        .get("tools")
        .and_then(|v| serde_json::from_value(v.clone()).ok())
        .unwrap_or_else(Vec::new);

    let subagent = spawner.spawn(purpose, tools).await
        .map_err(|e| MCPError {
            code: -32000,
            message: format!("Falha ao criar subagente: {}", e),
            data: None,
        })?;

    let response = json!({
        "id": subagent.identity.id,
        "purpose": subagent.identity.purpose,
        "parent_id": subagent.identity.parent_id,
        "created_at": subagent.identity.created_at.to_rfc3339(),
        "tools": subagent.tools,
        "is_active": subagent.is_active,
    });

    info!("📦 Subagente criado via MCP: {}", subagent.identity.id);
    Ok(response)
}

// ============================================================================
// 2. Handler: list_subagents
// ============================================================================

pub async fn handle_list_subagents(
    _args: serde_json::Value,
    state: &Arc<MCPServerState>,
) -> Result<serde_json::Value, MCPError> {
    let spawner = state.subagent_spawner.as_ref()
        .ok_or_else(|| MCPError {
            code: -32000,
            message: "SubagentSpawner não disponível".to_string(),
            data: None,
        })?;

    let subagents = spawner.list_active().await;
    let stats = spawner.stats().await;

    let list: Vec<serde_json::Value> = subagents.into_iter()
        .map(|s| json!({
            "id": s.identity.id,
            "purpose": s.identity.purpose,
            "parent_id": s.identity.parent_id,
            "created_at": s.identity.created_at.to_rfc3339(),
            "tools": s.tools,
            "is_active": s.is_active,
            "last_activity": s.last_activity.to_rfc3339(),
        }))
        .collect();

    Ok(json!({
        "total": stats.total,
        "active": stats.active,
        "purposes": stats.purposes,
        "subagents": list,
    }))
}

// ============================================================================
// 3. Handler: terminate_subagent
// ============================================================================

pub async fn handle_terminate_subagent(
    args: serde_json::Value,
    state: &Arc<MCPServerState>,
) -> Result<serde_json::Value, MCPError> {
    let spawner = state.subagent_spawner.as_ref()
        .ok_or_else(|| MCPError {
            code: -32000,
            message: "SubagentSpawner não disponível".to_string(),
            data: None,
        })?;

    let id = args
        .get("id")
        .and_then(|v| v.as_str())
        .ok_or_else(|| MCPError {
            code: -32602,
            message: "Campo 'id' obrigatório".to_string(),
            data: None,
        })?;

    let subagent = spawner.terminate(id).await
        .map_err(|e| MCPError {
            code: -32000,
            message: format!("Falha ao terminar subagente: {}", e),
            data: None,
        })?;

    Ok(json!({
        "id": subagent.identity.id,
        "status": "terminated",
        "terminated_at": chrono::Utc::now().to_rfc3339(),
    }))
}

// ============================================================================
// 4. Handler: execute_subagent
// ============================================================================

pub async fn handle_execute_subagent(
    args: serde_json::Value,
    state: &Arc<MCPServerState>,
) -> Result<serde_json::Value, MCPError> {
    let spawner = state.subagent_spawner.as_ref()
        .ok_or_else(|| MCPError {
            code: -32000,
            message: "SubagentSpawner não disponível".to_string(),
            data: None,
        })?;

    let id = args
        .get("id")
        .and_then(|v| v.as_str())
        .ok_or_else(|| MCPError {
            code: -32602,
            message: "Campo 'id' obrigatório".to_string(),
            data: None,
        })?;

    let task = args
        .get("task")
        .and_then(|v| v.as_str())
        .ok_or_else(|| MCPError {
            code: -32602,
            message: "Campo 'task' obrigatório".to_string(),
            data: None,
        })?;

    let cost_cap = args.get("cost_cap").and_then(|v| v.as_f64());

    let subagent = spawner.get(id).await
        .ok_or_else(|| MCPError {
            code: -32000,
            message: format!("Subagente '{}' não encontrado", id),
            data: None,
        })?;

    let provider = state.execution_provider.as_ref();

    let attestation = subagent.execute(task, provider, cost_cap).await
        .map_err(|e| MCPError {
            code: -32000,
            message: format!("Falha na execução: {}", e),
            data: None,
        })?;

    Ok(json!({
        "attestation_id": attestation.id,
        "task": task,
        "cost": attestation.cost_usd,
        "policy_compliance": attestation.is_policy_compliant(),
        "subagent_id": id,
    }))
}

// ============================================================================
// 5. Definições para tools/list
// ============================================================================

pub fn subagent_tool_definitions() -> Vec<serde_json::Value> {
    vec![
        json!({
            "name": "spawn_subagent",
            "description": "Cria um novo subagente com propósito específico",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "purpose": { "type": "string", "description": "Propósito do subagente" },
                    "tools": { "type": "array", "items": { "type": "string" }, "description": "Ferramentas permitidas" }
                },
                "required": ["purpose"]
            }
        }),
        json!({
            "name": "list_subagents",
            "description": "Lista todos os subagentes ativos",
            "inputSchema": { "type": "object", "properties": {} }
        }),
        json!({
            "name": "terminate_subagent",
            "description": "Termina um subagente pelo ID",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "id": { "type": "string", "description": "ID do subagente" }
                },
                "required": ["id"]
            }
        }),
        json!({
            "name": "execute_subagent",
            "description": "Executa uma tarefa em um subagente específico",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "id": { "type": "string" },
                    "task": { "type": "string" },
                    "cost_cap": { "type": "number" }
                },
                "required": ["id", "task"]
            }
        }),
    ]
}
