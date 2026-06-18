// src/integrations/picnic.rs
//! Integração com contratos Picnic (DeFi Basket) para gestão de royalties.
//! Suporta depósito de USDC em baskets e distribuição de shares/rendimentos.

use ethers::prelude::*;
use tracing::info;

// Geração do binding Rust para a interface do contrato Picnic Basket.
abigen!(
    IPicnicBasket,
    r#"[
        function deposit(uint256 amount, address receiver) external returns (uint256 shares)
        function totalAssets() external view returns (uint256)
        function distributeRewards(address[] calldata recipients, uint256[] calldata amounts) external
    ]"#
);

/// Gerencia a interação com um basket Picnic específico.
pub struct PicnicRoyaltyManager {
    /// Cliente middleware com signer para enviar transações.
    client: SignerMiddleware<Provider<Http>, Wallet<SigningKey>>,
    /// Endereço do contrato do basket.
    basket_address: Address,
    /// Endereço do USDC na rede (opcional, usado para validação).
    usdc_address: Option<Address>,
}

impl PicnicRoyaltyManager {
    /// Cria uma nova instância.
    ///
    /// # Parâmetros
    /// - `rpc_url`: URL do nó JSON-RPC (ex: Base, Polygon)
    /// - `private_key`: Chave privada da carteira que assinará as transações
    /// - `basket_address`: Endereço do contrato do basket Picnic
    /// - `usdc_address`: (opcional) Endereço do USDC para validação
    pub fn new(
        rpc_url: &str,
        private_key: &str,
        basket_address: Address,
        usdc_address: Option<Address>,
    ) -> Result<Self, String> {
        let provider = Provider::<Http>::try_from(rpc_url)
            .map_err(|e| format!("Erro ao conectar ao provider: {}", e))?;
        let wallet: LocalWallet = private_key
            .parse()
            .map_err(|e| format!("Chave privada inválida: {}", e))?;
        let client = SignerMiddleware::new(provider, wallet);

        Ok(Self {
            client,
            basket_address,
            usdc_address,
        })
    }

    /// Verifica se o basket está ativo e responde a chamadas.
    pub async fn verify_basket(&self) -> Result<(), String> {
        let contract = IPicnicBasket::new(self.basket_address, self.client.clone().into());
        let _total = contract.total_assets().call().await
            .map_err(|e| format!("Basket não responde ou inválido: {}", e))?;
        info!("✅ Basket Picnic verificado: {}", self.basket_address);
        Ok(())
    }

    /// Deposita USDC no basket e distribui shares/rendimentos conforme splits.
    pub async fn deposit_and_distribute(
        &self,
        amount_usdc: u64,
        splits: &[crate::evolution::desci_node_resource::RoyaltySplit],
    ) -> Result<TxHash, String> {
        if amount_usdc == 0 {
            return Err("Valor de depósito zero".to_string());
        }
        if splits.is_empty() {
            return Err("Nenhum split definido".to_string());
        }

        let contract = IPicnicBasket::new(self.basket_address, self.client.clone().into());

        // 1. Depositar USDC no basket
        info!("📤 Depositando {} USDC no basket...", amount_usdc);
        let deposit_tx = contract
            .deposit(U256::from(amount_usdc), self.client.address())
            .send()
            .await
            .map_err(|e| format!("Erro no depósito: {}", e))?;

        let receipt = deposit_tx.await
            .map_err(|e| format!("Erro ao confirmar depósito: {}", e))?
            .ok_or("Receipt is None")?;
        info!("✅ Depósito confirmado: tx={:?}", receipt.transaction_hash);

        // 2. Obter total de ativos para calcular as shares
        let total_assets = contract.total_assets().call().await
            .map_err(|e| format!("Erro ao obter totalAssets: {}", e))?;
        let total = total_assets.as_u64();

        if total == 0 {
            return Err("Total de ativos zero após depósito".to_string());
        }

        // 3. Construir listas de destinatários e valores (em shares ou tokens)
        let mut recipients = Vec::new();
        let mut amounts = Vec::new();

        for split in splits {
            let eth_addr = split.eth_address.as_ref()
                .ok_or_else(|| format!("Endereço Ethereum não definido para {}", split.npub))?;
            let addr: Address = eth_addr.parse()
                .map_err(|_| format!("Endereço Ethereum inválido: {}", eth_addr))?;

            let share = split.share.clamp(0.0, 1.0);
            let share_amount = (total as f64 * share as f64) as u64;
            if share_amount > 0 {
                recipients.push(addr);
                amounts.push(U256::from(share_amount));
            }
        }

        if recipients.is_empty() {
            return Err("Nenhum destinatário com share positiva".to_string());
        }

        // 4. Distribuir shares/rendimentos
        info!("📤 Distribuindo shares para {} destinatários...", recipients.len());
        let distribute_tx = contract
            .distribute_rewards(recipients, amounts)
            .send()
            .await
            .map_err(|e| format!("Erro na distribuição: {}", e))?;

        let receipt2 = distribute_tx.await
            .map_err(|e| format!("Erro ao confirmar distribuição: {}", e))?
            .ok_or("Receipt is None")?;
        info!("✅ Distribuição confirmada: tx={:?}", receipt2.transaction_hash);

        Ok(receipt2.transaction_hash)
    }

    /// Converte um endereço Ethereum (string) para Address.
    pub fn parse_address(addr: &str) -> Result<Address, String> {
        addr.parse().map_err(|_| format!("Endereço inválido: {}", addr))
    }
}