//! test-utils/src/fault_injection.rs
//! Wrappers para injetar falhas em replicação e rede.
//! Selo: CATHEDRAL-ARKHE-TEST-FAULT-INJECTION-v1.0.0

use anyhow::{anyhow, Result};
use arkhe_wormgraph::replication::{ReplicaStorage, VersionedEntry};
use async_trait::async_trait;
use rand::Rng;
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::RwLock;
use tokio::time::sleep;
use tracing::{debug, warn};

// ============================================================
// TIPO DE FALHA
// ============================================================

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FaultType {
    Timeout,
    NetworkError,
    DataCorruption,
    Latency(Duration),
    Partition,
}

impl FaultType {
    pub fn random_latency() -> Self {
        let ms = rand::thread_rng().gen_range(50..500);
        FaultType::Latency(Duration::from_millis(ms))
    }

    pub fn random_error() -> Self {
        let variants = vec![
            FaultType::Timeout,
            FaultType::NetworkError,
            FaultType::DataCorruption,
        ];
        variants[rand::thread_rng().gen_range(0..variants.len())]
    }
}

// ============================================================
// REPLICA STORAGE COM FALHAS
// ============================================================

pub struct FaultyReplicaStorage<T> {
    pub inner: Arc<dyn ReplicaStorage<T>>,
    node_id: String,
    fault_rate: f64,
    fault_type: FaultType,
    partition_visible: Arc<RwLock<Option<Vec<String>>>>,
    injected_faults: Arc<RwLock<u64>>,
}

impl<T: Clone + Send + Sync + 'static> FaultyReplicaStorage<T> {
    pub fn new(
        inner: Arc<dyn ReplicaStorage<T>>,
        node_id: &str,
        fault_rate: f64,
        fault_type: FaultType,
    ) -> Self {
        Self {
            inner,
            node_id: node_id.to_string(),
            fault_rate: fault_rate.clamp(0.0, 1.0),
            fault_type,
            partition_visible: Arc::new(RwLock::new(None)),
            injected_faults: Arc::new(RwLock::new(0)),
        }
    }

    pub async fn set_partition(&self, visible_peers: Option<Vec<String>>) {
        let mut guard = self.partition_visible.write().await;
        *guard = visible_peers;
        if let Some(peers) = &*guard {
            warn!("🔀 Nó {} em partição: visível para {:?}", self.node_id, peers);
        }
    }

    async fn maybe_inject_fault(&self) -> Result<()> {
        let mut rng = rand::thread_rng();
        if rng.gen_bool(self.fault_rate) {
            let fault = if let FaultType::Latency(_) = self.fault_type {
                FaultType::random_latency()
            } else if let FaultType::Partition { .. } = self.fault_type {
                FaultType::random_error()
            } else {
                self.fault_type
            };

            let mut count = self.injected_faults.write().await;
            *count += 1;

            match fault {
                FaultType::Timeout => {
                    debug!("⏱️ Nó {}: timeout injetado", self.node_id);
                    sleep(Duration::from_secs(5)).await;
                    return Err(anyhow!("Timeout simulado"));
                }
                FaultType::NetworkError => {
                    debug!("🌐 Nó {}: erro de rede injetado", self.node_id);
                    return Err(anyhow!("Erro de rede simulado"));
                }
                FaultType::DataCorruption => {
                    debug!("💾 Nó {}: corrupção de dados injetada", self.node_id);
                    return Err(anyhow!("Corrupção de dados simulada"));
                }
                FaultType::Latency(dur) => {
                    debug!("🐢 Nó {}: latência de {:?} injetada", self.node_id, dur);
                    sleep(dur).await;
                    Ok(())
                }
                FaultType::Partition { .. } => Ok(()),
            }
        } else {
            Ok(())
        }
    }

    pub async fn injected_fault_count(&self) -> u64 {
        *self.injected_faults.read().await
    }

    pub fn reset_faults(&self) {
        let mut count = self.injected_faults.blocking_write();
        *count = 0;
    }
}

#[async_trait]
impl<T: Clone + Send + Sync + 'static> ReplicaStorage<T> for FaultyReplicaStorage<T> {
    async fn read(&self, key: &str) -> Result<Option<VersionedEntry<T>>> {
        self.maybe_inject_fault().await?;
        self.inner.read(key).await
    }

    async fn write(&self, key: &str, entry: &VersionedEntry<T>) -> Result<()> {
        self.maybe_inject_fault().await?;
        self.inner.write(key, entry).await
    }

    async fn read_all(&self) -> Result<HashMap<String, VersionedEntry<T>>> {
        self.maybe_inject_fault().await?;
        self.inner.read_all().await
    }
}

// ============================================================
// FAULT INJECTION PARA QUORUM STORAGE (WRAPPER)
// ============================================================

pub struct FaultyQuorumStorage<T> {
    inner: Arc<arkhe_wormgraph::replication::QuorumStorage<T>>,
    _fault_nodes: Vec<String>,
    _fault_rate: f64,
}

impl<T: Clone + Send + Sync + 'static> FaultyQuorumStorage<T> {
    pub fn new(
        inner: Arc<arkhe_wormgraph::replication::QuorumStorage<T>>,
        fault_nodes: Vec<String>,
        fault_rate: f64,
    ) -> Self {
        Self {
            inner,
            _fault_nodes: fault_nodes,
            _fault_rate: fault_rate,
        }
    }

    pub async fn write_quorum_with_faults(
        &self,
        key: &str,
        entry: &VersionedEntry<T>,
    ) -> Result<()> {
        self.inner.write_quorum(key, entry).await
    }

    pub async fn read_quorum_with_faults(
        &self,
        key: &str,
    ) -> Result<Option<VersionedEntry<T>>> {
        self.inner.read_quorum(key).await
    }
}
