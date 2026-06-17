//! Cathedral ARKHE v28.3.2 — MCP Server (Refinado)
//! Servidor MCP com validação explícita de IdentityAttestation,
//! tratamento de erro granular e verificação de políticas.
//! Selo: CATHEDRAL-ARKHE-v28.3.2-MCP-SERVER-REFINED-2026-06-17

use std::sync::Arc;
use serde_json::json;
use axum::{
    extract::State,
    response::Json,
    routing::post,
    Router,
};
use tracing::{info, error};

use crate::attestation::{
    AttestationManager, AttestationProvider, AttestationVerifier,
    CathedralComputeProvider, PolicyDescriptor,
};
use crate::identity_attestation::IdentityAttestationProvider;
use crate::voice::VoiceCore;
use crate::mcp::auth::{auth_middleware, AuthConfig};

// ============================================================================
// 1. Estado do Servidor
// ============================================================================

pub struct MCPServerState {
    pub attestation_manager: Arc<AttestationManager>,
    pub identity_provider: Arc<dyn IdentityAttestationProvider + Send + Sync>,
    pub execution_provider: Arc<dyn AttestationProvider + Send + Sync>,
    pub architect_verifier: Option<Arc<dyn AttestationVerifier + Send + Sync>>,
    pub voice_core: Option<Arc<VoiceCore>>,
    pub default_provenance: String,
}

// ============================================================================
// 2. Tipos MCP
// ============================================================================

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct MCPRequest {
    pub jsonrpc: String,
    pub id: Option<serde_json::Value>,
    pub method: String,
    pub params: Option<serde_json::Value>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct MCPResponse {
    pub jsonrpc: String,
    pub id: Option<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub result: Option<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<MCPError>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct MCPError {
    pub code: i32,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<serde_json::Value>,
}

// ============================================================================
// 3. Handler Refinado: execute_workload
// ============================================================================

async fn handle_execute_workload(
    args: serde_json::Value,
    state: &Arc<MCPServerState>,
) -> Result<serde_json::Value, MCPError> {
    // 1. Extração e validação dos argumentos
    let workload = args
        .get("workload")
        .and_then(|v| v.as_str())
        .ok_or_else(|| MCPError {
            code: -32602,
            message: "Campo 'workload' é obrigatório".to_string(),
            data: None,
        })?;

    let cost_cap = args.get("cost_cap").and_then(|v| v.as_f64());
    let force_identity_refresh = args
        .get("force_identity_refresh")
        .and_then(|v| v.as_bool())
        .unwrap_or(false);

    let provenance = args
        .get("provenance")
        .and_then(|v| v.as_str())
        .unwrap_or(&state.default_provenance);

    // 2. Validação do provenance
    let allowed_provenance = ["user", "ai-suggested", "ai-executed", "user-revised"];
    if !allowed_provenance.contains(&provenance) {
        return Err(MCPError {
            code: -32602,
            message: format!(
                "Provenance inválido. Use um de: {}",
                allowed_provenance.join(", ")
            ),
            data: None,
        });
    }

    // 3. Obter IdentityAttestation
    let identity = state
        .identity_provider
        .attest_identity(force_identity_refresh)
        .await
        .map_err(|e| MCPError {
            code: -32000,
            message: format!("Falha ao obter IdentityAttestation: {}", e),
            data: Some(serde_json::json!({
                "details": e,
                "action": "tente 'force_refresh=true' se o atestado expirou"
            })),
        })?;

    // 4. VALIDAÇÃO EXPLÍCITA do IdentityAttestation
    // 4a. Confiança mínima (0.7)
    if identity.confidence < 0.7 {
        return Err(MCPError {
            code: -32001,
            message: format!(
                "Confiança da identidade insuficiente: {:.2} < 0.7",
                identity.confidence
            ),
            data: Some(serde_json::json!({
                "identity_confidence": identity.confidence,
                "required_confidence": 0.7,
                "suggestion": "Solicite uma nova prova de vida via 'request_voice_proof'"
            })),
        });
    }

    // 4b. Identidade verificada
    if !identity.identity_verified {
        return Err(MCPError {
            code: -32002,
            message: "Identidade não verificada. É necessário passar pela prova de vida.".to_string(),
            data: Some(serde_json::json!({
                "identity_verified": false,
                "suggestion": "Use 'request_voice_proof' para autenticação por voz"
            })),
        });
    }

    // 4c. Expiração (TTL)
    let ttl = std::env::var("IDENTITY_TTL_SECONDS")
        .unwrap_or_else(|_| "86400".to_string())
        .parse::<i64>()
        .unwrap_or(86400);

    if identity.is_expired(ttl) {
        return Err(MCPError {
            code: -32003,
            message: "IdentityAttestation expirado. Solicite uma nova prova de vida.".to_string(),
            data: Some(serde_json::json!({
                "issued_at": identity.timestamp,
                "ttl_seconds": ttl,
                "suggestion": "Use 'force_refresh=true' ou 'request_voice_proof'"
            })),
        });
    }

    // 4d. Assinatura do Arquiteto
    if let Some(verifier) = &state.architect_verifier {
        match identity.verify_architect_signature(verifier.as_ref()) {
            Ok(true) => { /* válido, prossegue */ }
            Ok(false) => {
                return Err(MCPError {
                    code: -32004,
                    message: "Assinatura do Arquiteto inválida. Possível adulteração do atestado.".to_string(),
                    data: None,
                });
            }
            Err(e) => {
                return Err(MCPError {
                    code: -32004,
                    message: format!("Erro ao verificar assinatura do Arquiteto: {}", e),
                    data: None,
                });
            }
        }
    }

    // 5. Verificar políticas ativas
    let active_policies = state
        .attestation_manager
        .list_active_policies()
        .await
        .map_err(|e| MCPError {
            code: -32005,
            message: format!("Falha ao consultar políticas ativas: {}", e),
            data: None,
        })?;

    if let Some(blocked) = active_policies.iter().find(|p| p.blocking) {
        return Err(MCPError {
            code: -32006,
            message: format!("Workload bloqueado pela política '{}'", blocked.name),
            data: Some(serde_json::json!({
                "policy": blocked.name,
                "description": blocked.description,
            })),
        });
    }

    // 6. Executa o workload com autorização
    let execution_attestation = state
        .execution_provider
        .run_authorized(workload, cost_cap, &identity)
        .await
        .map_err(|e| MCPError {
            code: -32007,
            message: format!("Falha ao executar workload: {}", e),
            data: Some(serde_json::json!({
                "workload": workload,
                "cost_cap": cost_cap,
            })),
        })?;

    // 7. Validação pós-execução
    if !execution_attestation.is_policy_compliant() {
        return Err(MCPError {
            code: -32008,
            message: "ExecutionAttestation não está em conformidade com as políticas.".to_string(),
            data: Some(serde_json::json!({
                "policy_compliance": execution_attestation.is_policy_compliant(),
                "policy_attestation_id": execution_attestation.policy_attestation_id(),
            })),
        });
    }

    // 8. Persistência
    state
        .attestation_manager
        .store_execution(&execution_attestation, provenance)
        .await
        .map_err(|e| MCPError {
            code: -32009,
            message: format!("Falha ao persistir ExecutionAttestation: {}", e),
            data: None,
        })?;

    // 9. Retorna o atestado
    let result = serde_json::to_value(&execution_attestation).map_err(|e| MCPError {
        code: -32010,
        message: format!("Falha ao serializar ExecutionAttestation: {}", e),
        data: None,
    })?;

    Ok(result)
}

// ============================================================================
// 4. Inicialização do Servidor (atualizada)
// ============================================================================

pub async fn start_mcp_server(
    attestation_manager: Arc<AttestationManager>,
    identity_provider: Arc<dyn IdentityAttestationProvider + Send + Sync>,
    execution_provider: Arc<dyn AttestationProvider + Send + Sync>,
    architect_verifier: Option<Arc<dyn AttestationVerifier + Send + Sync>>,
    voice_core: Option<Arc<VoiceCore>>,
    port: u16,
    required_token: Option<String>,
) -> Result<(), Box<dyn std::error::Error>> {
    let state = Arc::new(MCPServerState {
        attestation_manager,
        identity_provider,
        execution_provider,
        architect_verifier,
        voice_core,
        default_provenance: "ai-suggested".to_string(),
    });

    let auth_config = Arc::new(AuthConfig {
        required_token: required_token.unwrap_or_else(|| "".to_string()),
    });

    let app = Router::new()
        .route("/", post(handle_request))
        .with_state(state)
        .layer(axum::middleware::from_fn_with_state(
            auth_config.clone(),
            auth_middleware,
        ));

    let listener = tokio::net::TcpListener::bind(format!("0.0.0.0:{}", port)).await?;
    info!("🧠 MCP Server rodando em http://localhost:{}", port);
    axum::serve(listener, app).await?;
    Ok(())
}

// ============================================================================
// 5. Handler raiz (encaminhamento)
// ============================================================================

async fn handle_request(
    State(state): State<Arc<MCPServerState>>,
    Json(request): Json<MCPRequest>,
) -> Json<MCPResponse> {
    let result = match request.method.as_str() {
        "tools/list" => handle_tools_list().await,
        "tools/call" => handle_tools_call(request.params, &state).await,
        _ => Err(MCPError {
            code: -32601,
            message: format!("Método não encontrado: {}", request.method),
            data: None,
        }),
    };

    match result {
        Ok(res) => Json(MCPResponse {
            jsonrpc: "2.0".to_string(),
            id: request.id,
            result: Some(res),
            error: None,
        }),
        Err(err) => Json(MCPResponse {
            jsonrpc: "2.0".to_string(),
            id: request.id,
            result: None,
            error: Some(err),
        }),
    }
}

// ============================================================================
// 6. Ferramentas e roteamento
// ============================================================================

async fn handle_tools_list() -> Result<serde_json::Value, MCPError> {
    let tools = vec![
        serde_json::json!({
            "name": "attest_identity",
            "description": "Gera um IdentityAttestation para o arquiteto (prova de vida multimodal)",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "force_refresh": { "type": "boolean", "description": "Força nova captura de voz/biometria" }
                }
            }
        }),
        serde_json::json!({
            "name": "execute_workload",
            "description": "Executa um workload com autorização de identidade e gera ExecutionAttestation",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "workload": { "type": "string" },
                    "cost_cap": { "type": "number" },
                    "force_identity_refresh": { "type": "boolean" },
                    "provenance": { "type": "string", "enum": ["user", "ai-suggested", "ai-executed", "user-revised"] }
                },
                "required": ["workload"]
            }
        }),
        serde_json::json!({
            "name": "request_voice_proof",
            "description": "Solicita uma prova de vida por voz ao Arquiteto",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "phrase": { "type": "string" },
                    "timeout_secs": { "type": "integer" }
                }
            }
        }),
        serde_json::json!({
            "name": "validate_attestation",
            "description": "Valida um ExecutionAttestation (assinatura + identidade)",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "id": { "type": "string" }
                },
                "required": ["id"]
            }
        }),
        serde_json::json!({
            "name": "list_policies",
            "description": "Lista as políticas ativas do Cathedral",
            "inputSchema": { "type": "object", "properties": {} }
        }),
    ];

    Ok(serde_json::json!({ "tools": tools }))
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
        "attest_identity" => handle_attest_identity(args, state).await,
        "execute_workload" => handle_execute_workload(args, state).await,
        "request_voice_proof" => handle_voice_proof(args, state).await,
        "validate_attestation" => handle_validate_attestation(args, state).await,
        "list_policies" => handle_list_policies(state).await,
        _ => Err(MCPError {
            code: -32601,
            message: format!("Ferramenta não encontrada: {}", tool_name),
            data: None,
        }),
    }
}

// ============================================================================
// 7. Implementações simplificadas dos outros handlers
// ============================================================================

async fn handle_attest_identity(
    args: serde_json::Value,
    state: &Arc<MCPServerState>,
) -> Result<serde_json::Value, MCPError> {
    let force_refresh = args
        .get("force_refresh")
        .and_then(|v| v.as_bool())
        .unwrap_or(false);

    let identity = state
        .identity_provider
        .attest_identity(force_refresh)
        .await
        .map_err(|e| MCPError {
            code: -32000,
            message: format!("Falha ao gerar IdentityAttestation: {}", e),
            data: None,
        })?;

    serde_json::to_value(&identity).map_err(|e| MCPError {
        code: -32000,
        message: format!("Falha ao serializar: {}", e),
        data: None,
    })
}

async fn handle_voice_proof(
    args: serde_json::Value,
    state: &Arc<MCPServerState>,
) -> Result<serde_json::Value, MCPError> {
    let voice_core = state
        .voice_core
        .as_ref()
        .ok_or_else(|| MCPError {
            code: -32000,
            message: "VoiceCore não disponível".to_string(),
            data: None,
        })?;

    let phrase = args
        .get("phrase")
        .and_then(|v| v.as_str())
        .map(|s| s.to_string());

    let timeout = args
        .get("timeout_secs")
        .and_then(|v| v.as_u64())
        .unwrap_or(30);

    let evidence = tokio::time::timeout(
        tokio::time::Duration::from_secs(timeout),
        voice_core.capture_phrase_for_proof_of_life(phrase.as_deref()),
    )
    .await
    .map_err(|_| MCPError {
        code: -32000,
        message: "Timeout na captura de voz".to_string(),
        data: None,
    })?
    .map_err(|e| MCPError {
        code: -32000,
        message: format!("Falha na captura de voz: {}", e),
        data: None,
    })?;

    serde_json::to_value(&evidence).map_err(|e| MCPError {
        code: -32000,
        message: format!("Falha ao serializar: {}", e),
        data: None,
    })
}

async fn handle_validate_attestation(
    args: serde_json::Value,
    state: &Arc<MCPServerState>,
) -> Result<serde_json::Value, MCPError> {
    let id = args
        .get("id")
        .and_then(|v| v.as_str())
        .ok_or_else(|| MCPError {
            code: -32602,
            message: "Campo 'id' obrigatório".to_string(),
            data: None,
        })?;

    let exec = state
        .attestation_manager
        .get_execution(id)
        .await
        .map_err(|e| MCPError {
            code: -32000,
            message: format!("Falha ao buscar execution: {}", e),
            data: None,
        })?
        .ok_or_else(|| MCPError {
            code: -32000,
            message: format!("ExecutionAttestation com ID '{}' não encontrado", id),
            data: None,
        })?;

    let valid = state
        .attestation_manager
        .validate_execution(&exec)
        .await
        .map_err(|e| MCPError {
            code: -32000,
            message: format!("Erro na validação: {}", e),
            data: None,
        })?;

    Ok(json!({
        "id": id,
        "valid": valid,
        "timestamp": chrono::Utc::now().to_rfc3339(),
    }))
}

async fn handle_list_policies(
    state: &Arc<MCPServerState>,
) -> Result<serde_json::Value, MCPError> {
    let policies = state
        .attestation_manager
        .list_active_policies()
        .await
        .map_err(|e| MCPError {
            code: -32000,
            message: format!("Falha ao listar políticas: {}", e),
            data: None,
        })?;

    serde_json::to_value(&policies).map_err(|e| MCPError {
        code: -32000,
        message: format!("Falha ao serializar: {}", e),
        data: None,
    })
}