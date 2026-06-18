// src/integrations/x402.rs

use x402_axum::X402Middleware;
use x402_chain_eip155::V2Eip155Exact;
use x402_types::networks::USDC;
use bytes::Bytes;
use tracing::{info, error};
use ethers::prelude::*;
use crate::evolution::desci_node_resource::{RoyaltyConfig, RoyaltySplit};
use std::sync::Arc;

pub struct X402RoyaltyServer {
    pub middleware: X402Middleware,
    pub facilitator_url: String,
    pub provider: Arc<Provider<Http>>,              // Provider JSON-RPC (Base/Polygon)
    pub wallet: LocalWallet,                   // Wallet do servidor para assinar txs
}

impl X402RoyaltyServer {
    pub fn new(facilitator_url: &str, provider_url: &str, private_key: &str) -> Result<Self, String> {
        let provider = Provider::<Http>::try_from(provider_url)
            .map_err(|e| format!("Erro no provider: {}", e))?;
        let wallet: LocalWallet = private_key.parse().map_err(|e| format!("Chave invalida: {}", e))?;

        Ok(Self {
            middleware: X402Middleware::new(facilitator_url),
            facilitator_url: facilitator_url.to_string(),
            provider: Arc::new(provider),
            wallet,
        })
    }

    /// Converte npub para endereço Ethereum (usando a chave pública Nostr)
    pub fn npub_to_eth_address(&self, npub: &str) -> String {
        // Placeholder
        "0x0000000000000000000000000000000000000001".to_string()
    }

    /// Cria um middleware de proteção para um recurso com royalties
    pub fn protect_route(
        &self,
        royalty_config: &RoyaltyConfig,
    ) -> impl axum::middleware::Layer<()> {
        let price = royalty_config.price_per_access
            .split_whitespace()
            .next()
            .unwrap_or("0.001")
            .parse::<u64>()
            .unwrap_or(1000) * 10_000; // 0.001 USDC = 1000 (6 decimals) → ajuste

        let receiver = royalty_config.royalty_split
            .first()
            .map(|s| self.npub_to_eth_address(&s.npub))
            .unwrap_or_else(|| "0x0000000000000000000000000000000000000000".to_string());

        // Cria um price tag para a rede Base (USDC)
        let price_tag = V2Eip155Exact::price_tag(
            receiver.parse().unwrap(),
            USDC::base().amount(price),
        );

        self.middleware.clone().with_price_tag(price_tag)
    }

    pub async fn settle_payment_with_picnic(
        &self,
        payment_amount: u64,                   // em USDC (6 decimals)
        royalty_splits: &[RoyaltySplit],
        picnic_basket_address: &str,
    ) -> Result<(), String> {
        let basket_addr: Address = picnic_basket_address.parse().map_err(|_| "Endereço inválido")?;

        let client = SignerMiddleware::new(self.provider.clone(), self.wallet.clone());
        let picnic_manager = crate::integrations::picnic::PicnicRoyaltyManager::new(
            "mock_url",
            &std::env::var("PRIVATE_KEY").unwrap_or_default(),
            basket_addr,
            None
        )?;

        picnic_manager.deposit_and_distribute(payment_amount, royalty_splits).await.map(|_| ())
    }

    pub async fn verify_basket(&self, _address: &Address) -> Result<(), String> {
        Ok(())
    }
}

/// Cliente x402 para agentes (compradores)
pub struct X402Client {
    pub client: reqwest::Client,
}

impl X402Client {
    pub fn new() -> Self {
        Self {
            client: reqwest::Client::new(),
        }
    }

    /// Baixa um recurso pagando via x402 (fluxo automático)
    pub async fn download_with_payment(
        &self,
        url: &str,
        wallet_private_key: &str,
    ) -> Result<Bytes, String> {
        // Passo 1: GET inicial → recebe 402
        let response = self.client.get(url).send().await
            .map_err(|e| format!("Erro na requisição: {}", e))?;

        if response.status() != reqwest::StatusCode::PAYMENT_REQUIRED {
            // Se não exigir pagamento, retorna o conteúdo diretamente
            return response.bytes().await.map_err(|e| e.to_string());
        }

        // Passo 2: Extrair instruções de pagamento do header ou body
        let payment_instructions = response.headers()
            .get("x-402-payment")
            .and_then(|v| v.to_str().ok())
            .ok_or("Instruções de pagamento não encontradas")?;

        // Passo 3: Assinar pagamento com a wallet
        let signature = self.sign_payment(payment_instructions, wallet_private_key);

        // Passo 4: Reenviar com a assinatura
        let final_response = self.client
            .get(url)
            .header("PAYMENT-SIGNATURE", signature)
            .send()
            .await
            .map_err(|e| format!("Erro no pagamento: {}", e))?;

        if final_response.status().is_success() {
            final_response.bytes().await.map_err(|e| e.to_string())
        } else {
            Err("Falha no pagamento".to_string())
        }
    }

    fn sign_payment(&self, _instructions: &str, _private_key: &str) -> String {
        // Em produção: usar biblioteca de assinatura (ex: secp256k1)
        "signed_payment".to_string()
    }
}