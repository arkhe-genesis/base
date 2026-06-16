use async_trait::async_trait;
use ethers::prelude::*;
use std::sync::Arc;
use tokio::sync::mpsc;
use tracing::{info, error};

use crate::governance::bindings::CathedralConsensusLedger;

pub struct LedgerEvent {
    pub event_type: String,
    pub payload: String,
    pub timestamp: u64,
    pub policy_version: u64,
    pub signature: Option<String>,
}

#[async_trait]
pub trait ConsensusLedger: Send + Sync {
    async fn record_event(&self, event: LedgerEvent) -> Result<(), String>;
}

// Relayer completo com ethers + fila
pub struct EthersRelayer {
    sender: mpsc::Sender<LedgerEvent>,
}

impl EthersRelayer {
    pub async fn new<M: Middleware + 'static>(
        client: Arc<M>,
        contract_address: Address,
    ) -> Result<Self, String> {
        let (tx, mut rx) = mpsc::channel::<LedgerEvent>(100);
        let contract = CathedralConsensusLedger::new(contract_address, client);

        tokio::spawn(async move {
            while let Some(event) = rx.recv().await {
                let sig_bytes = if let Some(sig_hex) = event.signature {
                    match hex::decode(&sig_hex) {
                        Ok(b) => ethers::core::types::Bytes::from(b),
                        Err(_) => ethers::core::types::Bytes::default(),
                    }
                } else {
                    ethers::core::types::Bytes::default()
                };

                let call = contract.record_event(
                    event.event_type,
                    event.payload,
                    event.policy_version,
                    sig_bytes,
                );

                let pending_tx = call.send().await;
                match pending_tx {
                    Ok(pending_tx) => {
                        info!("Tx sent: {:?}", pending_tx.tx_hash());
                        match pending_tx.await {
                            Ok(Some(receipt)) => {
                                info!("Tx confirmed in block {:?}", receipt.block_number);
                            }
                            Ok(None) => error!("Tx dropped"),
                            Err(e) => error!("Error waiting for tx: {:?}", e),
                        }
                    }
                    Err(e) => error!("Error sending tx: {:?}", e),
                }
            }
        });

        Ok(Self {
            sender: tx,
        })
    }
}

#[async_trait]
impl ConsensusLedger for EthersRelayer {
    async fn record_event(&self, event: LedgerEvent) -> Result<(), String> {
        self.sender.send(event).await.map_err(|e| format!("Channel error: {}", e))
    }
}
