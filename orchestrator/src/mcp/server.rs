//! Cathedral ARKHE v28.3.2 — MCP Server (Com SubagentSpawner)
//! Servidor MCP com suporte a subagentes, contexto e identidade soberana.
//! Selo: CATHEDRAL-ARKHE-v28.3.2-MCP-SERVER-FINAL-2026-06-17

use std::sync::Arc;
use serde_json::json;
use axum::{extract::State, response::Json, routing::post, Router};
use tracing::{info, error};

use crate::mcp::subagent_handler::*;
use crate::mcp::context_handler::*;
use crate::mcp::voice_handler::*;

use crate::subagent_spawner::SubagentSpawner;
use crate::context::ContextManager;
use crate::attestation::{AttestationManager, IdentityAttestationProvider, AttestationProvider, AttestationVerifier};

// Stub out these types for now as they are missing from the mock orchestrator module
pub struct VoiceCore {}
pub struct BlockchainNervousSystem {}

#[derive(Debug, Clone, serde::Serialize)]
pub struct MCPError {
    pub code: i32,
    pub message: String,
    pub data: Option<serde_json::Value>,
}

// ============================================================================
// 1. Estado do Servidor (com SubagentSpawner)
// ============================================================================

pub struct MCPServerState {
    pub attestation_manager: Arc<AttestationManager>,
    pub identity_provider: Arc<dyn IdentityAttestationProvider + Send + Sync>,
    pub execution_provider: Arc<dyn AttestationProvider + Send + Sync>,
    pub architect_verifier: Option<Arc<dyn AttestationVerifier + Send + Sync>>,
    pub voice_core: Option<Arc<VoiceCore>>,
    pub nervous_system: Option<Arc<BlockchainNervousSystem>>,
    pub subagent_spawner: Option<Arc<SubagentSpawner>>, // ✅ NOVO
    pub context_manager: Option<Arc<ContextManager>>,
    pub default_provenance: String,
}

// ============================================================================
// 2. Inicialização do Servidor
// ============================================================================

pub async fn start_mcp_server(
    attestation_manager: Arc<AttestationManager>,
    identity_provider: Arc<dyn IdentityAttestationProvider + Send + Sync>,
    execution_provider: Arc<dyn AttestationProvider + Send + Sync>,
    architect_verifier: Option<Arc<dyn AttestationVerifier + Send + Sync>>,
    voice_core: Option<Arc<VoiceCore>>,
    nervous_system: Option<Arc<BlockchainNervousSystem>>,
    subagent_spawner: Option<Arc<SubagentSpawner>>, // ✅ NOVO
    context_manager: Option<Arc<ContextManager>>,
    port: u16,
) -> Result<(), Box<dyn std::error::Error>> {
    let state = Arc::new(MCPServerState {
        attestation_manager,
        identity_provider,
        execution_provider,
        architect_verifier,
        voice_core,
        nervous_system,
        subagent_spawner,
        context_manager,
        default_provenance: "ai-suggested".to_string(),
    });

    let app = Router::new()
        .route("/", post(handle_request))
        .with_state(state);

    let listener = tokio::net::TcpListener::bind(format!("0.0.0.0:{}", port)).await?;
    info!("🧠 MCP Server rodando em http://localhost:{}", port);
    axum::serve(listener, app).await?;
    Ok(())
}

// ============================================================================
// 3. Handlers (Subagent + Context)
// ============================================================================

async fn handle_request(
    State(state): State<Arc<MCPServerState>>,
    axum::extract::Json(payload): axum::extract::Json<serde_json::Value>,
) -> Json<serde_json::Value> {
    let method = payload.get("method").and_then(|v| v.as_str()).unwrap_or("");

    let result = match method {
        "tools/list" => handle_tools_list().await,
        "tools/call" => handle_tools_call(payload.get("params").cloned(), &state).await,
        _ => Err(MCPError {
            code: -32601,
            message: format!("Método não suportado: {}", method),
            data: None,
        }),
    };

    match result {
        Ok(res) => Json(json!({
            "jsonrpc": "2.0",
            "result": res,
            "id": payload.get("id")
        })),
        Err(err) => Json(json!({
            "jsonrpc": "2.0",
            "error": err,
            "id": payload.get("id")
        })),
    }
}

// handle_tools_list e handle_tools_call devem incluir todas as ferramentas
async fn handle_tools_list() -> Result<serde_json::Value, MCPError> {
    let mut tools = vec![
        // attest_identity_tool_definition(),
        // execute_workload_tool_definition(),
        voice_proof_tool_definition(),
        // validate_attestation_tool_definition(),
        // list_policies_tool_definition(),
    ];
    tools.extend(subagent_tool_definitions()); // ✅ 4 ferramentas de subagente
    tools.extend(context_tool_definitions());  // ✅ 5 ferramentas de contexto
    Ok(json!({ "tools": tools }))
}

async fn handle_tools_call(
    params: Option<serde_json::Value>,
    state: &Arc<MCPServerState>,
) -> Result<serde_json::Value, MCPError> {
    let params = params.ok_or_else(|| MCPError {
        code: -32602,
        message: "Parâmetros ausentes".to_string(),
        data: None,
    })?;

    let tool_name = params
        .get("name")
        .and_then(|v| v.as_str())
        .ok_or_else(|| MCPError {
            code: -32602,
            message: "Campo 'name' obrigatório".to_string(),
            data: None,
        })?;

    let args = params
        .get("arguments")
        .cloned()
        .unwrap_or(serde_json::Value::Null);

    match tool_name {
        "request_voice_proof" => handle_request_voice_proof(args, state).await,
        "spawn_subagent" => handle_spawn_subagent(args, state).await,
        "list_subagents" => handle_list_subagents(args, state).await,
        "terminate_subagent" => handle_terminate_subagent(args, state).await,
        "execute_subagent" => handle_execute_subagent(args, state).await,
        "create_context" => handle_create_context(args, state).await,
        "add_to_context" => handle_add_to_context(args, state).await,
        "get_context" => handle_get_context(args, state).await,
        "list_contexts" => handle_list_contexts(args, state).await,
        "clear_context" => handle_clear_context(args, state).await,
        _ => Err(MCPError {
            code: -32601,
            message: format!("Ferramenta não encontrada: {}", tool_name),
            data: None,
        }),
    }
}
