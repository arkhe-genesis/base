pub mod attestation;
pub mod identity_attestation;
pub mod mcp;
pub mod voice;

use std::sync::Arc;
use tracing::{info, error};

use crate::attestation::{AttestationManager, CathedralComputeProvider, AttestationProvider, AttestationVerifier};
use crate::identity_attestation::IdentityAttestationProvider;
use crate::voice::VoiceCore;
use crate::mcp::server::start_mcp_server;

// Placeholder structs to make the code compile conceptually
pub struct ArchitectSigner {}
pub struct NervousSystem {}
pub struct EventStore {}
pub struct IdentityProvider {}
impl IdentityAttestationProvider for IdentityProvider {
    fn attest_identity(&self, _force_refresh: bool) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<crate::identity_attestation::IdentityAttestation, String>> + Send>> {
        Box::pin(async { Ok(crate::identity_attestation::IdentityAttestation {
            confidence: 1.0,
            identity_verified: true,
            timestamp: 0,
        }) })
    }
}
pub struct Verifier {}
impl AttestationVerifier for Verifier {}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt::init();

    // Placeholder instantiations
    let attestation_manager = Arc::new(AttestationManager {});
    let voice_core = Arc::new(VoiceCore {});
    let architect_signer = Arc::new(ArchitectSigner {});
    let nervous_system = Arc::new(NervousSystem {});
    let event_store = Arc::new(EventStore {});
    let identity_provider = Arc::new(IdentityProvider {});
    let architect_verifier = Arc::new(Verifier {});

    // 1. Configuração do MCP
    let mcp_enabled = std::env::var("ENABLE_MCP_SERVER")
        .unwrap_or_else(|_| "false".to_string())
        .parse::<bool>()
        .unwrap_or(false);

    if mcp_enabled {
        let mcp_port = std::env::var("MCP_PORT")
            .unwrap_or_else(|_| "3032".to_string())
            .parse::<u16>()
            .unwrap_or(3032);

        let mcp_token = std::env::var("MCP_AUTH_TOKEN").ok();

        // 2. Cria execution provider (CathedralComputeProvider)
        let execution_provider: Arc<dyn AttestationProvider + Send + Sync> =
            Arc::new(CathedralComputeProvider::new(
                architect_signer.clone(),
                nervous_system.clone(),
                event_store.clone(),
                "cathedral-v1",
            ));

        // 3. Verificador (pode ser opcional)
        let architect_verifier_opt: Option<Arc<dyn AttestationVerifier + Send + Sync>> =
            Some(architect_verifier.clone());

        // 4. Inicia o servidor
        let attestation_manager_clone = attestation_manager.clone();
        let identity_provider_clone = identity_provider.clone();
        let execution_provider_clone = execution_provider.clone();
        let voice_core_clone = Some(voice_core.clone());

        tokio::spawn(async move {
            if let Err(e) = start_mcp_server(
                attestation_manager_clone,
                identity_provider_clone,
                execution_provider_clone,
                                architect_verifier_opt,
                voice_core_clone,
                None,
                None,
                None,
                mcp_port,
                mcp_token
            )
            .await
            {
                error!("❌ MCP Server falhou: {}", e);
            }
        });

        info!("🧠 MCP Server iniciado na porta {}", mcp_port);
    }

    // Keep the main thread alive for demonstration purposes
    tokio::time::sleep(tokio::time::Duration::from_secs(60)).await;

    Ok(())
}