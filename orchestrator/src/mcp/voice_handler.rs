use std::sync::Arc;
use serde_json::json;

use crate::mcp::server::{MCPError, MCPServerState};

pub fn voice_proof_tool_definition() -> serde_json::Value {
    json!({
        "name": "request_voice_proof",
        "description": "Requests a voice proof of life",
        "inputSchema": {
            "type": "object",
            "properties": {
                "user_id": { "type": "string" }
            },
            "required": ["user_id"]
        }
    })
}

pub async fn handle_request_voice_proof(
    _args: serde_json::Value,
    _state: &Arc<MCPServerState>,
) -> Result<serde_json::Value, MCPError> {
    Ok(json!({ "status": "simulated_success" }))
}
