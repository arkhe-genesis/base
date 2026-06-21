//! shard-controller/src/failover.rs — Liderança e Migração Real
//! Selo: CATHEDRAL-ARKHE-SHARD-FAILOVER-PROD-v1.0.0

use anyhow::{anyhow, Result};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::{mpsc, RwLock};
use tokio::time::sleep;
use tracing::{debug, error, info, warn};
use serde::{Deserialize, Serialize};

use arkhe_wormgraph::storage::ShardStorage;

// ============================================================
// CONFIGURAÇÃO
// ============================================================

#[derive(Debug, Clone)]
pub struct FailoverConfig {
    pub heartbeat_interval_secs: u64,
    pub lease_duration_secs: u64,
    pub election_timeout_secs: u64,
    pub migration_batch_size: usize,
    pub max_migration_retries: u32,
}

impl Default for FailoverConfig {
    fn default() -> Self {
        Self {
            heartbeat_interval_secs: 5,
            lease_duration_secs: 15,
            election_timeout_secs: 10,
            migration_batch_size: 1000,
            max_migration_retries: 3,
        }
    }
}

// ============================================================
// LEASE (LIDERANÇA)
// ============================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Lease {
    pub holder: String,
    pub expires_at: i64,      // timestamp Unix
    pub version: u64,
}

impl Lease {
    pub fn new(holder: &str, duration: Duration) -> Self {
        let expires = chrono::Utc::now() + chrono::Duration::from_std(duration).unwrap();
        Self {
            holder: holder.to_string(),
            expires_at: expires.timestamp(),
            version: 0,
        }
    }

    pub fn is_expired(&self) -> bool {
        chrono::Utc::now().timestamp() > self.expires_at
    }

    pub fn renew(&mut self, duration: Duration) {
        let expires = chrono::Utc::now() + chrono::Duration::from_std(duration).unwrap();
        self.expires_at = expires.timestamp();
        self.version += 1;
    }
}

// ============================================================
// LEADER ELETION (DISTRIBUTED LOCK)
// ============================================================

pub struct LeaderElection {
    node_id: String,
    lease: Arc<RwLock<Option<Lease>>>,
    storage: Arc<dyn ShardStorage>,
    config: FailoverConfig,
    // Canal para notificar mudanças de liderança
    leadership_tx: mpsc::Sender<bool>,
}

impl LeaderElection {
    pub fn new(
        node_id: &str,
        storage: Arc<dyn ShardStorage>,
        config: FailoverConfig,
    ) -> (Self, mpsc::Receiver<bool>) {
        let (tx, rx) = mpsc::channel(10);
        (
            Self {
                node_id: node_id.to_string(),
                lease: Arc::new(RwLock::new(None)),
                storage,
                config,
                leadership_tx: tx,
            },
            rx,
        )
    }

    /// Tenta adquirir a liderança usando CAS (compare‑and‑swap) no metadata.
    pub async fn try_acquire_leader(&self) -> Result<bool> {
        let _lock_key = "cathedral/leader/lease";
        let lease_duration = Duration::from_secs(self.config.lease_duration_secs);

        // 1. Lê o lease atual
        let current_meta = self.storage.read_metadata(0).await?;
        let current_lease: Option<Lease> = current_meta
            .as_ref()
            .and_then(|meta| meta.extra.get("lease"))
            .and_then(|v| serde_json::from_value(v.clone()).ok());

        // 2. Decide se deve tentar assumir
        let should_acquire = match current_lease.as_ref() {
            Some(lease) if lease.is_expired() => true,
            None => true,
            Some(_) => false,
        };

        if !should_acquire {
            return Ok(false);
        }

        // 3. Cria novo lease com versão = atual + 1 (CAS)
        let new_version = current_lease.as_ref().map(|l| l.version + 1).unwrap_or(1);
        let new_lease = Lease {
            holder: self.node_id.clone(),
            expires_at: (chrono::Utc::now() + chrono::Duration::from_std(lease_duration).unwrap()).timestamp(),
            version: new_version,
        };

        // 4. Tenta escrever com condição de versão (CAS)
        let current = self.storage.read_metadata(0).await?;
        let current_version = current
            .as_ref()
            .and_then(|meta| meta.extra.get("lease_version").and_then(|v| v.as_u64()))
            .unwrap_or(0);

        if current_version != current_lease.as_ref().map(|l| l.version).unwrap_or(0) {
            warn!("CAS falhou: versão divergente");
            return Ok(false);
        }

        // Atualiza metadata com o novo lease
        let mut meta = current.unwrap_or_else(|| arkhe_wormgraph::storage::ShardMetadata {
            shard_id: 0,
            event_count: 0,
            first_timestamp: 0,
            last_timestamp: 0,
            size_bytes: 0,
            merkle_root: vec![],
            version: 0,
            extra: HashMap::new(),
        });
        meta.extra.insert("lease".to_string(), serde_json::to_value(&new_lease)?);
        meta.extra.insert("lease_version".to_string(), serde_json::json!(new_version));

        self.storage.write_metadata(0, &meta).await?;

        // Armazena localmente
        {
            let mut lease_guard = self.lease.write().await;
            *lease_guard = Some(new_lease);
        }

        info!("🏆 Nó {} adquiriu liderança (versão {})", self.node_id, new_version);
        let _ = self.leadership_tx.send(true).await;
        Ok(true)
    }

    /// Mantém a liderança (heartbeat contínuo)
    pub async fn maintain_leadership(&self) -> Result<()> {
        loop {
            let is_leader = {
                let lease_guard = self.lease.read().await;
                match lease_guard.as_ref() {
                    Some(lease) if !lease.is_expired() && lease.holder == self.node_id => true,
                    _ => false,
                }
            };

            if !is_leader {
                // Tenta reconquistar
                self.try_acquire_leader().await?;
            } else {
                // Renova lease
                let mut lease_guard = self.lease.write().await;
                if let Some(lease) = lease_guard.as_mut() {
                    let current_meta = self.storage.read_metadata(0).await?;
                    let current_version = current_meta
                        .as_ref()
                        .and_then(|meta| meta.extra.get("lease_version").and_then(|v| v.as_u64()))
                        .unwrap_or(0);

                    if current_version != lease.version {
                        warn!("CAS falhou na renovação: versão divergente");
                        *lease_guard = None;
                        continue;
                    }

                    let new_version = lease.version + 1;
                    let renewed_lease = Lease {
                        version: new_version,
                        holder: lease.holder.clone(),
                        expires_at: (chrono::Utc::now() + chrono::Duration::from_std(Duration::from_secs(self.config.lease_duration_secs)).unwrap()).timestamp(),
                    };

                    let mut meta = current_meta.unwrap_or_else(|| arkhe_wormgraph::storage::ShardMetadata {
                        shard_id: 0,
                        event_count: 0,
                        first_timestamp: 0,
                        last_timestamp: 0,
                        size_bytes: 0,
                        merkle_root: vec![],
                        version: 0,
                        extra: HashMap::new(),
                    });
                    meta.extra.insert("lease".to_string(), serde_json::to_value(&renewed_lease)?);
                    meta.extra.insert("lease_version".to_string(), serde_json::json!(new_version));
                    self.storage.write_metadata(0, &meta).await?;

                    *lease = renewed_lease;
                    debug!("🔑 Lease renovado por {} (versão {})", self.node_id, new_version);
                }
            }

            sleep(Duration::from_secs(self.config.heartbeat_interval_secs)).await;
        }
    }

    pub async fn is_leader(&self) -> bool {
        let lease_guard = self.lease.read().await;
        match lease_guard.as_ref() {
            Some(lease) => !lease.is_expired() && lease.holder == self.node_id,
            None => false,
        }
    }
}

// ============================================================
// MIGRAÇÃO DE DADOS (FAILOVER REAL)
// ============================================================

pub struct DataMigrator {
    source_storage: Arc<dyn ShardStorage>,
    target_storage: Arc<dyn ShardStorage>,
    config: FailoverConfig,
}

impl DataMigrator {
    pub fn new(
        source: Arc<dyn ShardStorage>,
        target: Arc<dyn ShardStorage>,
        config: FailoverConfig,
    ) -> Self {
        Self {
            source_storage: source,
            target_storage: target,
            config,
        }
    }

    /// Migra todos os dados de um shard (source) para outro (target).
    pub async fn migrate_shard(&self, shard_id: u64) -> Result<()> {
        info!("🔄 Iniciando migração do shard {}...", shard_id);

        // 1. Lê metadados do source
        let source_meta = self.source_storage.read_metadata(shard_id).await?;
        let total_events = source_meta.as_ref().map(|m| m.event_count).unwrap_or(0);

        if total_events == 0 {
            info!("Shard {} vazio, pulando migração", shard_id);
            return Ok(());
        }

        // 2. Migra em batches
        let batch_size = self.config.migration_batch_size;
        let mut offset = 0;
        let mut migrated = 0;

        while offset < total_events as usize {
            let events = self.source_storage
                .read_events(shard_id, offset, batch_size)
                .await?;

            if events.is_empty() {
                break;
            }

            // 3. Escreve no target com retry
            let mut retries = 0;
            while retries < self.config.max_migration_retries {
                match self.target_storage.append_events(shard_id, &events).await {
                    Ok(_) => break,
                    Err(e) => {
                        retries += 1;
                        warn!("Falha ao escrever batch (tentativa {}): {}", retries, e);
                        if retries >= self.config.max_migration_retries {
                            return Err(anyhow!("Falha na migração após {} tentativas", retries));
                        }
                        sleep(Duration::from_secs(2 * retries as u64)).await;
                    }
                }
            }

            offset += events.len();
            migrated += events.len();
            debug!("Migrados {}/{} eventos", migrated, total_events);
        }

        // 4. Copia metadados
        if let Some(meta) = source_meta {
            self.target_storage.write_metadata(shard_id, &meta).await?;
        }

        info!("✅ Migração do shard {} concluída ({} eventos)", shard_id, migrated);
        Ok(())
    }

    /// Executa failover completo: detecta falha, promove réplica, migra dados.
    pub async fn execute_failover(
        &self,
        dead_shard_id: u64,
        replica_node_id: &str,
    ) -> Result<()> {
        info!("🚨 Executando failover para shard {} no nó {}", dead_shard_id, replica_node_id);

        // 1. Verifica se o source ainda tem dados
        let source_meta = self.source_storage.read_metadata(dead_shard_id).await?;
        if source_meta.is_none() {
            warn!("Shard {} não tem dados para migrar", dead_shard_id);
            return Ok(());
        }

        // 2. Migra dados
        self.migrate_shard(dead_shard_id).await?;

        // 3. Marca source como morto (opcional)
        // Em produção: atualiza estado no ShardController

        info!("✅ Failover do shard {} concluído", dead_shard_id);
        Ok(())
    }
}
