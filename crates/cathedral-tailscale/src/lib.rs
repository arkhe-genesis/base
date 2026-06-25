pub mod client;
pub mod derp;
pub mod grants;
pub mod headscale;
pub mod metrics;
pub mod psk;

use anyhow::Error;
use cathedral_identity::{Did, VerifiableCredential};

/// Wrapper principal para integração Tailscale/Headscale
pub struct CathedralTailscale {
    headscale: headscale::HeadscaleClient,
    psk_manager: psk::PskManager,
    metrics: metrics::TailscaleMetrics,
}

impl CathedralTailscale {
    /// Inicializa conexão com Headscale + autenticação DID
    pub async fn connect(
        &self,
        did: &Did,
        credential: &VerifiableCredential,
    ) -> Result<client::TailnetConnection, Error> {
        let start = std::time::Instant::now();
        // 1. Verificar credencial DID
        credential.verify()?;

        // 2. Obter PSK para esta conexão
        let psk = self.psk_manager.get_or_create(did).await?;

        // 3. Autenticar no Headscale via OIDC bridge
        let identity = self.headscale.authenticate(did, credential).await?;

        // 4. Configurar WireGuard com PSK
        let wg_config = client::WireGuardConfig::new().with_psk(&psk).with_identity(&identity);

        // 5. Registrar métricas
        self.metrics.handshake_latency.observe(start.elapsed().as_secs_f64());

        Ok(client::TailnetConnection::new(wg_config))
    }
}
