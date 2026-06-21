//! src/substrato_4004/policy_adapter.rs
//! Adapter para o PolicyRegistry da Base

use ethers::{
    abi::Abi,
    contract::Contract,
    providers::{Http, Provider},
    types::Address,
};

use crate::substrato_4004::b20_mapper::PolicyScope;

#[derive(Debug)]
pub enum PolicyError {
    ContractError(String),
}

/// Cliente para o PolicyRegistry singleton da Base
pub struct PolicyAdapter {
    contract: Contract<Provider<Http>>,
    #[allow(dead_code)]
    b20_factory: Address,
}

impl PolicyAdapter {
    pub fn new(
        client: std::sync::Arc<Provider<Http>>,
        address: Address,
        #[allow(dead_code)] b20_factory: Address,
    ) -> Self {
        // Minimal valid ABI with methods called
        let abi_str = r#"[
            {"inputs":[{"internalType":"address","name":"admin","type":"address"},{"internalType":"uint8","name":"policyType","type":"uint8"},{"internalType":"address[]","name":"initialAccounts","type":"address[]"}],"name":"createPolicyWithAccounts","outputs":[{"internalType":"uint64","name":"","type":"uint64"}],"stateMutability":"nonpayable","type":"function"},
            {"inputs":[{"internalType":"uint64","name":"policyId","type":"uint64"},{"internalType":"address","name":"account","type":"address"}],"name":"isAuthorized","outputs":[{"internalType":"bool","name":"","type":"bool"}],"stateMutability":"view","type":"function"},
            {"inputs":[{"internalType":"uint64","name":"policyId","type":"uint64"},{"internalType":"bool","name":"block_","type":"bool"},{"internalType":"address[]","name":"accounts","type":"address[]"}],"name":"updateBlocklist","outputs":[],"stateMutability":"nonpayable","type":"function"}
        ]"#;
        let abi: Abi = serde_json::from_str(abi_str).unwrap();
        let contract = Contract::new(address, abi, client);
        Self { contract, b20_factory }
    }

    /// Cria uma nova policy
    pub async fn create_policy(
        &self,
        admin: Address,
        policy_type: PolicyType,
        initial_accounts: Vec<Address>,
    ) -> Result<u64, PolicyError> {
        let tx = self
            .contract
            .method::<_, u64>(
                "createPolicyWithAccounts",
                (admin, policy_type as u8, initial_accounts),
            )
            .map_err(|e| PolicyError::ContractError(e.to_string()))?;

        let _receipt = tx.send().await.map_err(|e| PolicyError::ContractError(e.to_string()))?;

        // Need to parse logs to get actual policy id in real world, but returning 0 for now as specified
        Ok(0)
    }

    /// Verifica se conta e autorizada sob uma policy
    pub async fn is_authorized(
        &self,
        policy_id: u64,
        account: Address,
    ) -> Result<bool, PolicyError> {
        let authorized: bool = self
            .contract
            .method("isAuthorized", (policy_id, account))
            .map_err(|e| PolicyError::ContractError(e.to_string()))?
            .call()
            .await
            .map_err(|e| PolicyError::ContractError(e.to_string()))?;

        Ok(authorized)
    }

    /// Atualiza blocklist (batched)
    pub async fn update_blocklist(
        &self,
        policy_id: u64,
        block: bool,
        accounts: Vec<Address>,
    ) -> Result<(), PolicyError> {
        let tx = self
            .contract
            .method::<_, ()>("updateBlocklist", (policy_id, block, accounts))
            .map_err(|e| PolicyError::ContractError(e.to_string()))?;

        let _receipt = tx.send().await.map_err(|e| PolicyError::ContractError(e.to_string()))?;

        Ok(())
    }

    /// Obtem policy ID para um scope de um token B20
    pub async fn get_policy(&self, token: Address, scope: PolicyScope) -> Result<u64, PolicyError> {
        // B20 token call
        let abi_str = r#"[{"inputs":[{"internalType":"uint8","name":"scope","type":"uint8"}],"name":"policyId","outputs":[{"internalType":"uint64","name":"","type":"uint64"}],"stateMutability":"view","type":"function"}]"#;
        let abi: Abi = serde_json::from_str(abi_str).unwrap();
        let b20 = Contract::new(token, abi, self.contract.client());

        let policy_id: u64 = b20
            .method("policyId", scope as u8)
            .map_err(|e| PolicyError::ContractError(e.to_string()))?
            .call()
            .await
            .map_err(|e| PolicyError::ContractError(e.to_string()))?;

        Ok(policy_id)
    }
}

#[derive(Debug, Clone, Copy)]
#[repr(u8)]
pub enum PolicyType {
    Blocklist = 0,
    Allowlist = 1,
}
