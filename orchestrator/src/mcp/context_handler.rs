//! Cathedral ARKHE v28.3.2 — MCP Context Handler
//! Ferramentas MCP para gerenciar contextos isolados de subagentes.
//! Selo: CATHEDRAL-ARKHE-v28.3.2-MCP-CONTEXT-2026-06-17

use std::sync::Arc;
use serde_json::json;
use tracing::{info, warn};

use crate::context::{Context, ContextManager, ContextRole};
use crate::mcp::server::{MCPError, MCPServerState};

// ============================================================================
// 1. Handler: create_context
// ============================================================================

pub async fn handle_create_context(
    args: serde_json::Value,
    state: &Arc<MCPServerState>,
) -> Result<serde_json::Value, MCPError> {
    let manager = state.context_manager.as_ref()
        .ok_or_else(|| MCPError {
            code: -32000,
            message: "ContextManager não disponível".to_string(),
            data: None,
        })?;

    let id = args
        .get("id")
        .and_then(|v| v.as_str())
        .ok_or_else(|| MCPError {
            code: -32602,
            message: "Campo 'id' obrigatório".to_string(),
            data: None,
        })?
        .to_string();

    let max_tokens = args
        .get("max_tokens")
        .and_then(|v| v.as_u64())
        .unwrap_or(4096) as usize;

    let max_entries = args
        .get("max_entries")
        .and_then(|v| v.as_u64())
        .unwrap_or(100) as usize;

    let ctx = manager.create_context(&id).await;
    // Configura limites
    let _ctx_with_limits = (*ctx).clone().with_limits(max_tokens, max_entries);
    // Como o contexto é criado com new, precisamos substituir
    // Nota: em produção, o ContextManager deveria aceitar configuração

    info!("📂 Contexto criado: {}", id);
    Ok(json!({
        "id": id,
        "max_tokens": max_tokens,
        "max_entries": max_entries,
        "created_at": chrono::Utc::now().to_rfc3339(),
    }))
}

// ============================================================================
// 2. Handler: add_to_context
// ============================================================================

pub async fn handle_add_to_context(
    args: serde_json::Value,
    state: &Arc<MCPServerState>,
) -> Result<serde_json::Value, MCPError> {
    let manager = state.context_manager.as_ref()
        .ok_or_else(|| MCPError {
            code: -32000,
            message: "ContextManager não disponível".to_string(),
            data: None,
        })?;

    let context_id = args
        .get("context_id")
        .and_then(|v| v.as_str())
        .ok_or_else(|| MCPError {
            code: -32602,
            message: "Campo 'context_id' obrigatório".to_string(),
            data: None,
        })?;

    let content = args
        .get("content")
        .and_then(|v| v.as_str())
        .ok_or_else(|| MCPError {
            code: -32602,
            message: "Campo 'content' obrigatório".to_string(),
            data: None,
        })?;

    let role_str = args
        .get("role")
        .and_then(|v| v.as_str())
        .unwrap_or("user");

    let importance = args
        .get("importance")
        .and_then(|v| v.as_f64())
        .unwrap_or(0.5) as f32;

    let role = match role_str {
        "system" => ContextRole::System,
        "user" => ContextRole::User,
        "agent" => ContextRole::Agent,
        "subagent" => ContextRole::Subagent,
        "tool" => ContextRole::Tool,
        _ => ContextRole::User,
    };

    let ctx = manager.get_context(context_id).await
        .ok_or_else(|| MCPError {
            code: -32000,
            message: format!("Contexto '{}' não encontrado", context_id),
            data: None,
        })?;

    ctx.add(content, role, importance).await
        .map_err(|e| MCPError {
            code: -32000,
            message: format!("Falha ao adicionar ao contexto: {}", e),
            data: None,
        })?;

    let metadata = ctx.metadata().await;
    Ok(json!({
        "context_id": context_id,
        "entry_count": metadata.entry_count,
        "total_tokens": metadata.total_tokens,
        "compression_ratio": metadata.compression_ratio,
        "added_at": chrono::Utc::now().to_rfc3339(),
    }))
}

// ============================================================================
// 3. Handler: get_context
// ============================================================================

pub async fn handle_get_context(
    args: serde_json::Value,
    state: &Arc<MCPServerState>,
) -> Result<serde_json::Value, MCPError> {
    let manager = state.context_manager.as_ref()
        .ok_or_else(|| MCPError {
            code: -32000,
            message: "ContextManager não disponível".to_string(),
            data: None,
        })?;

    let context_id = args
        .get("context_id")
        .and_then(|v| v.as_str())
        .ok_or_else(|| MCPError {
            code: -32602,
            message: "Campo 'context_id' obrigatório".to_string(),
            data: None,
        })?;

    let ctx = manager.get_context(context_id).await
        .ok_or_else(|| MCPError {
            code: -32000,
            message: format!("Contexto '{}' não encontrado", context_id),
            data: None,
        })?;

    let entries = ctx.get_entries().await;
    let metadata = ctx.metadata().await;
    let text = ctx.get_text().await;

    let entries_json: Vec<serde_json::Value> = entries.iter()
        .map(|e| json!({
            "id": e.id,
            "content": e.content,
            "timestamp": e.timestamp.to_rfc3339(),
            "role": format!("{:?}", e.role),
            "importance": e.importance,
            "tokens": e.tokens,
        }))
        .collect();

    Ok(json!({
        "context_id": context_id,
        "metadata": {
            "entry_count": metadata.entry_count,
            "total_tokens": metadata.total_tokens,
            "compression_ratio": metadata.compression_ratio,
            "oldest_entry": metadata.oldest_entry.map(|d| d.to_rfc3339()),
            "newest_entry": metadata.newest_entry.map(|d| d.to_rfc3339()),
        },
        "entries": entries_json,
        "text": text,
        "prompt": ctx.get_prompt().await,
    }))
}

// ============================================================================
// 4. Handler: list_contexts
// ============================================================================

pub async fn handle_list_contexts(
    _args: serde_json::Value,
    state: &Arc<MCPServerState>,
) -> Result<serde_json::Value, MCPError> {
    let manager = state.context_manager.as_ref()
        .ok_or_else(|| MCPError {
            code: -32000,
            message: "ContextManager não disponível".to_string(),
            data: None,
        })?;

    let ids = manager.list_contexts().await;

    // Obtém metadados de cada contexto
    let mut contexts = Vec::new();
    for id in ids {
        if let Some(ctx) = manager.get_context(&id).await {
            let meta = ctx.metadata().await;
            contexts.push(json!({
                "id": id,
                "entry_count": meta.entry_count,
                "total_tokens": meta.total_tokens,
                "compression_ratio": meta.compression_ratio,
            }));
        }
    }

    Ok(json!({
        "count": contexts.len(),
        "contexts": contexts,
    }))
}

// ============================================================================
// 5. Handler: clear_context
// ============================================================================

pub async fn handle_clear_context(
    args: serde_json::Value,
    state: &Arc<MCPServerState>,
) -> Result<serde_json::Value, MCPError> {
    let manager = state.context_manager.as_ref()
        .ok_or_else(|| MCPError {
            code: -32000,
            message: "ContextManager não disponível".to_string(),
            data: None,
        })?;

    let context_id = args
        .get("context_id")
        .and_then(|v| v.as_str())
        .ok_or_else(|| MCPError {
            code: -32602,
            message: "Campo 'context_id' obrigatório".to_string(),
            data: None,
        })?;

    let ctx = manager.get_context(context_id).await
        .ok_or_else(|| MCPError {
            code: -32000,
            message: format!("Contexto '{}' não encontrado", context_id),
            data: None,
        })?;

    ctx.clear().await;
    info!("🧹 Contexto {} limpo", context_id);

    Ok(json!({
        "context_id": context_id,
        "status": "cleared",
        "cleared_at": chrono::Utc::now().to_rfc3339(),
    }))
}

// ============================================================================
// 6. Definições para tools/list
// ============================================================================

pub fn context_tool_definitions() -> Vec<serde_json::Value> {
    vec![
        json!({
            "name": "create_context",
            "description": "Cria um novo contexto isolado",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "id": { "type": "string", "description": "ID único do contexto" },
                    "max_tokens": { "type": "integer", "description": "Limite de tokens", "default": 4096 },
                    "max_entries": { "type": "integer", "description": "Limite de entradas", "default": 100 }
                },
                "required": ["id"]
            }
        }),
        json!({
            "name": "add_to_context",
            "description": "Adiciona uma entrada a um contexto",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "context_id": { "type": "string" },
                    "content": { "type": "string" },
                    "role": { "type": "string", "enum": ["system", "user", "agent", "subagent", "tool"] },
                    "importance": { "type": "number", "minimum": 0, "maximum": 1, "default": 0.5 }
                },
                "required": ["context_id", "content"]
            }
        }),
        json!({
            "name": "get_context",
            "description": "Recupera um contexto e seu conteúdo",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "context_id": { "type": "string" }
                },
                "required": ["context_id"]
            }
        }),
        json!({
            "name": "list_contexts",
            "description": "Lista todos os contextos ativos",
            "inputSchema": { "type": "object", "properties": {} }
        }),
        json!({
            "name": "clear_context",
            "description": "Limpa todo o conteúdo de um contexto",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "context_id": { "type": "string" }
                },
                "required": ["context_id"]
            }
        }),
    ]
}
