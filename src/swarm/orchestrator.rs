// src/swarm/orchestrator.rs

use crate::integrations::picnic::PicnicRoyaltyManager;
use crate::evolution::desci_node_resource::{RoyaltyConfig, RoyaltySplit, FreeTier, DeSciNodeResource};
use crate::integrations::x402::{X402RoyaltyServer, X402Client};
use bytes::Bytes;

pub struct IdentityManager {
    pub npub: String,
    pub metadata: IdentityMetadata,
}

pub struct IdentityMetadata {
    pub author: String,
}

impl IdentityManager {
    pub fn get_orcid_by_npub(&self, _npub: &str) -> Option<String> {
        None
    }

    pub fn add_provenance(&self, _action: &str, _author: &str, _desc: &str, _a: Option<&str>, _b: Option<&str>) {
    }
}

pub struct Config {
    pub rpc_url: String,
    pub private_key: String,
}

pub struct SecondSelfOrchestrator {
    pub x402_server: X402RoyaltyServer,
    pub x402_client: X402Client,
    pub identity: IdentityManager,
    pub config: Config,
    pub base_url: String,
}

impl SecondSelfOrchestrator {
    pub fn new() -> Self {
        let facilitator_url = std::env::var("X402_FACILITATOR_URL")
            .unwrap_or_else(|_| "https://api.x402.org/v1".to_string());

        let server = X402RoyaltyServer::new(&facilitator_url, "http://localhost:8545", &std::env::var("PRIVATE_KEY").unwrap_or_default()).unwrap();

        Self {
            x402_server: server,
            x402_client: X402Client::new(),
            identity: IdentityManager {
                npub: "mock_npub".to_string(),
                metadata: IdentityMetadata { author: "mock_author".to_string() }
            },
            config: Config {
                rpc_url: "mock_url".to_string(),
                private_key: "0x0".to_string(),
            },
            base_url: "http://localhost".to_string(),
        }
    }

    pub fn get_desci_node_mut(&mut self, node_id: &str) -> Option<DeSciNodeResource> {
        Some(DeSciNodeResource::new(node_id, "mock_title", "mock_npub", None))
    }

    pub fn get_desci_node(&self, node_id: &str) -> Option<DeSciNodeResource> {
        Some(DeSciNodeResource::new(node_id, "mock_title", "mock_npub", None))
    }

    pub async fn load_desci_node(&self, node_id: &str) -> Result<DeSciNodeResource, String> {
        Ok(DeSciNodeResource::new(node_id, "mock_title", "mock_npub", None))
    }

    pub async fn save_node_version(&self, _node: &mut DeSciNodeResource) -> Result<(), String> {
        Ok(())
    }

    pub async fn save_node_version_by_val(&self, _node: DeSciNodeResource) -> Result<(), String> {
        Ok(())
    }

    pub async fn publish_desci_node(&self, _node: &mut DeSciNodeResource, _publish: bool) -> Result<String, String> {
        Ok("mock_dpid".to_string())
    }

    pub async fn enable_royalties(
        &mut self,
        node_id: &str,
        price: &str,
        splits: Vec<(String, f32)>,
        picnic_basket: Option<&str>,      // ← NOVO: endereço do basket Picnic
        free_tier: Option<FreeTier>,
    ) -> Result<(), String> {
        let mut node = self.get_desci_node_mut(node_id)
            .ok_or_else(|| format!("Node {} não encontrado", node_id))?;

        let now = chrono::Utc::now().timestamp() as u64;

        // Converter splits para RoyaltySplit (com ORCID e endereço Ethereum)
        let royalty_splits: Vec<RoyaltySplit> = splits.into_iter()
            .map(|(npub, share)| {
                let orcid = self.identity.get_orcid_by_npub(&npub);
                let eth_address = self.x402_server.npub_to_eth_address(&npub);
                RoyaltySplit {
                    npub,
                    share,
                    orcid,
                    eth_address: Some(eth_address),
                    pix_key: None,
                }
            })
            .collect();

        // Validar soma dos shares = 1.0
        let total_share: f32 = royalty_splits.iter().map(|s| s.share).sum();
        if (total_share - 1.0).abs() > 0.001 {
            return Err("A soma das participações deve ser 1.0".to_string());
        }

        // Validar e verificar basket Picnic se fornecido
        let basket_address = if let Some(basket) = picnic_basket {
            let addr = basket.parse()
                .map_err(|_| "Endereço do basket inválido".to_string())?;
            // Verificar se o basket existe e responde
            let picnic_manager = PicnicRoyaltyManager::new(
                &self.config.rpc_url,
                &self.config.private_key,
                addr,
                None,
            ).map_err(|e| format!("Erro ao conectar ao Picnic: {}", e))?;
            picnic_manager.verify_basket().await?;
            Some(basket.to_string())
        } else {
            None
        };

        // Atualizar configuração do node
        node.royalty_config = Some(RoyaltyConfig {
            enabled: true,
            price_per_access: price.to_string(),
            currency: "USDC".to_string(),
            chain: "eip155:8453".to_string(), // Base
            royalty_split: royalty_splits,
            free_tier,
            picnic_basket: basket_address,
            created_at: now,
            updated_at: now,
        });

        // Registra o middleware x402 (se o servidor HTTP estiver ativo)
        if let Some(config) = node.royalty_config.as_ref() {
            let _layer = self.x402_server.protect_route(config);
        }

        // Persiste no HashTree
        self.save_node_version_by_val(node).await?;

        // Registra proveniência
        self.identity.add_provenance(
            "enable_royalties",
            &self.identity.metadata.author,
            &format!("Royalties configurados para Node {} (basket: {:?})", node_id, picnic_basket),
            None,
            Some(&format!("{} USDC", price)),
        );

        Ok(())
    }

    pub async fn download_desci_component(
        &self,
        dpid: &str,
        component_id: &str,
        wallet_private_key: &str,
    ) -> Result<Bytes, String> {
        let node = self.get_desci_node(dpid)
            .ok_or_else(|| format!("Node {} não encontrado", dpid))?;

        let url = format!("{}/desci/{}/components/{}", self.base_url, dpid, component_id);

        if let Some(royalty) = &node.royalty_config {
            if royalty.enabled {
                // 1. Paga via x402
                let payment = self.x402_client.download_with_payment(&url, wallet_private_key).await?;

                // 2. Envia USDC para o Picnic Basket
                if let Some(basket) = &royalty.picnic_basket {
                    self.x402_server.settle_payment_with_picnic(
                        royalty.price_per_access.split_whitespace().next().unwrap_or("0.001").parse::<f64>().unwrap_or(0.001) as u64 * 1000000,
                        &royalty.royalty_split,
                        basket
                    ).await?;
                }

                // 3. Registra o acesso no HashTree
                self.record_access(dpid, &payment).await?;

                return Ok(payment);
            }
        }

        // Sem royalties: download direto
        let bytes = self.get_component_data(dpid, component_id).await?;
        Ok(Bytes::from(bytes))
    }

    pub async fn distribute_via_pix(
        &mut self,
        recipients: Vec<(String, f64)>, // (pix_key, amount_brl)
    ) -> Result<(), String> {
        // Usa Open Finance para transferir Pix
        let openfinance = crate::integrations::pix::OpenFinanceClient::new(
            &std::env::var("OPENFINANCE_URL").unwrap_or_default(),
            &std::env::var("OPENFINANCE_CLIENT_ID").unwrap_or_default(),
            &std::env::var("OPENFINANCE_CLIENT_SECRET").unwrap_or_default(),
        );

        // Obtém consentimento (em produção: usar consentimento do criador)
        let consent = self.get_openfinance_consent().await?;

        for (pix_key, amount) in recipients {
            if amount > 0.0 {
                openfinance.transfer_pix(&consent, &pix_key, amount, "Royalties ARKHE").await?;
                tracing::info!("💸 Pix enviado para {}: BRL {:.2}", pix_key, amount);
            }
        }

        Ok(())
    }

    pub async fn get_brl_usdc_rate(&self) -> Result<f64, String> {
        Ok(5.70)
    }

    pub async fn get_openfinance_consent(&self) -> Result<crate::integrations::pix::OpenFinanceConsent, String> {
        Ok(crate::integrations::pix::OpenFinanceConsent {
            consent_id: "mock".to_string(),
            access_token: "mock".to_string(),
            refresh_token: "mock".to_string(),
            expires_at: chrono::Utc::now(),
            scope: vec![],
        })
    }

    pub async fn record_access(&self, _dpid: &str, _payment: &[u8]) -> Result<(), String> {
        Ok(())
    }

    pub async fn get_component_data(&self, _dpid: &str, _component_id: &str) -> Result<Vec<u8>, String> {
        Ok(vec![])
    }
}