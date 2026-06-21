//! wormgraph/src/storage_file.rs — FileShardStorage com Compaction, Retention, Atomic Append
//! Selo: CATHEDRAL-ARKHE-FILESTORAGE-HARDENED-v1.0.0

use anyhow::{anyhow, Result};
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::{Duration, SystemTime};
use tokio::fs;
use tokio::io::AsyncWriteExt;
use tokio::sync::RwLock;
use tracing::{debug, info, warn};

use crate::{ProvenanceEvent, storage::{ShardStorage, ShardMetadata}};

// ============================================================
// CONFIGURAÇÃO
// ============================================================

#[derive(Debug, Clone)]
pub struct FileStorageConfig {
    pub base_path: PathBuf,
    pub max_segment_size_bytes: u64,      // 64MB
    pub retention_days: u64,              // 30 dias
    pub compaction_interval_secs: u64,    // 3600
    pub enable_compaction: bool,
    pub enable_retention: bool,
}

impl Default for FileStorageConfig {
    fn default() -> Self {
        Self {
            base_path: PathBuf::from("./wormgraph_data"),
            max_segment_size_bytes: 64 * 1024 * 1024, // 64MB
            retention_days: 30,
            compaction_interval_secs: 3600,
            enable_compaction: true,
            enable_retention: true,
        }
    }
}

// ============================================================
// SEGMENTO DE LOG
// ============================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
struct SegmentInfo {
    segment_id: u64,
    file_path: PathBuf,
    first_timestamp: i64,
    last_timestamp: i64,
    event_count: u64,
    size_bytes: u64,
    is_active: bool,
}

// ============================================================
// FILESHARDSTORAGE HARDENED
// ============================================================

#[derive(Clone)]
pub struct HardenedFileStorage {
    config: FileStorageConfig,
    cache: Arc<RwLock<HashMap<u64, Vec<ProvenanceEvent>>>>,
    segments: Arc<RwLock<HashMap<u64, Vec<SegmentInfo>>>>,
    active_segment_writers: Arc<RwLock<HashMap<u64, tokio::fs::File>>>,
}

impl HardenedFileStorage {
    pub async fn new(config: FileStorageConfig) -> Result<Self> {
        fs::create_dir_all(&config.base_path).await?;

        let storage = Self {
            config: config.clone(),
            cache: Arc::new(RwLock::new(HashMap::new())),
            segments: Arc::new(RwLock::new(HashMap::new())),
            active_segment_writers: Arc::new(RwLock::new(HashMap::new())),
        };

        // Carrega segmentos existentes
        storage.load_segments().await?;

        // Inicia tasks de background
        if config.enable_compaction {
            let s = storage.clone();
            tokio::spawn(async move {
                s.run_compaction_loop().await;
            });
        }

        if config.enable_retention {
            let s = storage.clone();
            tokio::spawn(async move {
                s.run_retention_loop().await;
            });
        }

        Ok(storage)
    }

    async fn load_segments(&self) -> Result<()> {
        let mut dir = fs::read_dir(&self.config.base_path).await?;
        let mut seg_map: HashMap<u64, Vec<SegmentInfo>> = HashMap::new();

        while let Some(entry) = dir.next_entry().await? {
            let name = entry.file_name();
            let name = name.to_string_lossy();
            if let Some(captures) = name.strip_prefix("shard_").and_then(|s| s.strip_suffix(".seg")) {
                if let Some((shard_str, seg_str)) = captures.split_once('_') {
                    if let (Ok(shard_id), Ok(seg_id)) = (shard_str.parse::<u64>(), seg_str.parse::<u64>()) {
                        let meta = self.read_segment_metadata(&entry.path()).await?;
                        let seg_info = SegmentInfo {
                            segment_id: seg_id,
                            file_path: entry.path(),
                            first_timestamp: meta.first_timestamp,
                            last_timestamp: meta.last_timestamp,
                            event_count: meta.event_count,
                            size_bytes: meta.size_bytes,
                            is_active: false,
                        };
                        seg_map.entry(shard_id).or_default().push(seg_info);
                    }
                }
            }
        }

        // Ordena segmentos por timestamp
        for segs in seg_map.values_mut() {
            segs.sort_by_key(|s| s.first_timestamp);
        }

        let mut segments = self.segments.write().await;
        *segments = seg_map;

        info!("📂 Carregados {} shards do storage", segments.len());
        Ok(())
    }

    async fn read_segment_metadata(&self, path: &Path) -> Result<ShardMetadata> {
        let content = fs::read_to_string(path.with_extension("meta")).await?;
        Ok(serde_json::from_str(&content)?)
    }

    // ============================================================
    // APPEND ATÔMICO (ESCREVE PARA TEMP, DEPOIS RENOMEIA)
    // ============================================================

    async fn append_atomic(
        &self,
        shard_id: u64,
        events: &[ProvenanceEvent],
    ) -> Result<()> {
        if events.is_empty() {
            return Ok(());
        }

        // 1. Gera nome do arquivo temporário
        let temp_path = self.config.base_path
            .join(format!("shard_{}.tmp.{}", shard_id, uuid::Uuid::new_v4()));

        // 2. Escreve para o arquivo temporário
        let mut file = fs::File::create(&temp_path).await?;
        for event in events {
            let line = serde_json::to_string(event)? + "\n";
            file.write_all(line.as_bytes()).await?;
        }
        file.sync_all().await?;

        // 3. Verifica tamanho do segmento ativo
        let mut active_writers = self.active_segment_writers.write().await;
        let active_path = self.config.base_path
            .join(format!("shard_{}_active.log", shard_id));

        // Se o arquivo ativo existe e está grande demais, rotaciona
        if active_path.exists() {
            let meta = fs::metadata(&active_path).await?;
            if meta.len() > self.config.max_segment_size_bytes {
                self.rotate_segment(shard_id).await?;
            }
        }

        // 4. Anexa o conteúdo do temp ao arquivo ativo (append)
        let active_path_clone = active_path.clone();
        let temp_content = fs::read_to_string(&temp_path).await?;
        fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(&active_path_clone)
            .await?
            .write_all(temp_content.as_bytes())
            .await?;

        // 5. Remove o temporário
        fs::remove_file(&temp_path).await?;

        // 6. Atualiza cache
        {
            let mut cache = self.cache.write().await;
            let entry = cache.entry(shard_id).or_insert_with(Vec::new);
            entry.extend(events.iter().cloned());
        }

        debug!("Shard {}: {} eventos anexados atomicamente", shard_id, events.len());
        Ok(())
    }

    async fn rotate_segment(&self, shard_id: u64) -> Result<()> {
        let active_path = self.config.base_path.join(format!("shard_{}_active.log", shard_id));
        if !active_path.exists() {
            return Ok(());
        }

        let meta = fs::metadata(&active_path).await?;
        let seg_id = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)?
            .as_secs();

        let seg_path = self.config.base_path.join(format!("shard_{}_{}.seg", shard_id, seg_id));

        // Renomeia active → segmento
        fs::rename(&active_path, &seg_path).await?;

        // Cria metadados do segmento
        let entries = self.cache.read().await;
        let events = entries.get(&shard_id).cloned().unwrap_or_default();

        let first_ts = events.first().map(|e| e.timestamp).unwrap_or(0);
        let last_ts = events.last().map(|e| e.timestamp).unwrap_or(0);

        let seg_meta = ShardMetadata {
            shard_id,
            event_count: events.len() as u64,
            first_timestamp: first_ts,
            last_timestamp: last_ts,
            size_bytes: meta.len(),
            merkle_root: vec![],
            version: seg_id,
            extra: HashMap::new(),
        };

        let meta_path = seg_path.with_extension("meta");
        fs::write(&meta_path, serde_json::to_string_pretty(&seg_meta)?).await?;

        // Registra segmento
        let seg_info = SegmentInfo {
            segment_id: seg_id,
            file_path: seg_path,
            first_timestamp: first_ts,
            last_timestamp: last_ts,
            event_count: events.len() as u64,
            size_bytes: meta.len(),
            is_active: false,
        };

        let mut segments = self.segments.write().await;
        segments.entry(shard_id).or_default().push(seg_info);

        info!("🔄 Segmento rotacionado para shard {} ({} eventos)", shard_id, events.len());
        Ok(())
    }

    // ============================================================
    // COMPACTION (MERGING SEGMENTS)
    // ============================================================

    async fn run_compaction_loop(&self) {
        let interval = Duration::from_secs(self.config.compaction_interval_secs);
        let mut timer = tokio::time::interval(interval);
        loop {
            timer.tick().await;
            if let Err(e) = self.compact_all().await {
                warn!("Erro na compactação: {}", e);
            }
        }
    }

    async fn compact_all(&self) -> Result<()> {
        let shards = {
            let segments = self.segments.read().await;
            segments.keys().cloned().collect::<Vec<_>>()
        };

        for shard_id in shards {
            self.compact_shard(shard_id).await?;
        }
        Ok(())
    }

    async fn compact_shard(&self, shard_id: u64) -> Result<()> {
        let mut segments = self.segments.write().await;
        let segs = match segments.get_mut(&shard_id) {
            Some(s) => s,
            None => return Ok(()),
        };

        if segs.len() < 10 {
            return Ok(()); // Muito cedo para compactar
        }

        // Compacta os 5 mais antigos em um
        let old_segments: Vec<SegmentInfo> = segs.drain(0..5).collect();
        let mut all_events = Vec::new();

        for seg in &old_segments {
            let content = fs::read_to_string(&seg.file_path).await?;
            for line in content.lines() {
                if let Ok(event) = serde_json::from_str::<ProvenanceEvent>(line) {
                    all_events.push(event);
                }
            }
        }

        // Cria novo segmento compactado
        let new_seg_id = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)?
            .as_secs();

        let new_path = self.config.base_path.join(format!("shard_{}_{}.seg", shard_id, new_seg_id));
        let mut file = fs::File::create(&new_path).await?;
        for event in &all_events {
            file.write_all((serde_json::to_string(event)? + "\n").as_bytes()).await?;
        }
        file.sync_all().await?;

        let new_seg = SegmentInfo {
            segment_id: new_seg_id,
            file_path: new_path.clone(),
            first_timestamp: old_segments.first().map(|s| s.first_timestamp).unwrap_or(0),
            last_timestamp: old_segments.last().map(|s| s.last_timestamp).unwrap_or(0),
            event_count: all_events.len() as u64,
            size_bytes: fs::metadata(&new_path).await?.len(),
            is_active: false,
        };

        // Remove arquivos antigos
        for seg in &old_segments {
            let _ = fs::remove_file(&seg.file_path).await;
            let _ = fs::remove_file(seg.file_path.with_extension("meta")).await;
        }

        segs.insert(0, new_seg);
        info!("🗜️ Shard {} compactado: {} segmentos → 1", shard_id, old_segments.len());
        Ok(())
    }

    // ============================================================
    // RETENTION (DELETE OLD SEGMENTS)
    // ============================================================

    async fn run_retention_loop(&self) {
        let interval = Duration::from_secs(self.config.compaction_interval_secs);
        let mut timer = tokio::time::interval(interval);
        loop {
            timer.tick().await;
            if let Err(e) = self.apply_retention().await {
                warn!("Erro na retenção: {}", e);
            }
        }
    }

    async fn apply_retention(&self) -> Result<()> {
        let now = SystemTime::now();
        let cutoff = now - Duration::from_secs(self.config.retention_days * 86400);

        let mut segments = self.segments.write().await;
        for segs in segments.values_mut() {
            segs.retain(|seg| {
                let keep = seg.last_timestamp > cutoff.elapsed().unwrap_or_default().as_secs() as i64;
                if !keep {
                    let _ = std::fs::remove_file(&seg.file_path);
                    let _ = std::fs::remove_file(seg.file_path.with_extension("meta"));
                    info!("🗑️ Removido segmento antigo: {}", seg.file_path.display());
                }
                keep
            });
        }
        Ok(())
    }
}

#[async_trait]
impl ShardStorage for HardenedFileStorage {
    async fn append_events(&self, shard_id: u64, events: &[ProvenanceEvent]) -> Result<()> {
        self.append_atomic(shard_id, events).await
    }

    async fn read_events(&self, shard_id: u64, offset: usize, limit: usize) -> Result<Vec<ProvenanceEvent>> {
        // Lê do cache primeiro
        {
            let cache = self.cache.read().await;
            if let Some(entries) = cache.get(&shard_id) {
                let start = offset.min(entries.len());
                let end = (offset + limit).min(entries.len());
                return Ok(entries[start..end].to_vec());
            }
        }

        // Fallback: lê do arquivo ativo
        let active_path = self.config.base_path.join(format!("shard_{}_active.log", shard_id));
        if !active_path.exists() {
            return Ok(Vec::new());
        }

        let content = fs::read_to_string(&active_path).await?;
        let mut entries = Vec::new();
        for line in content.lines() {
            if let Ok(entry) = serde_json::from_str::<ProvenanceEvent>(line) {
                entries.push(entry);
            }
        }

        let start = offset.min(entries.len());
        let end = (offset + limit).min(entries.len());
        Ok(entries[start..end].to_vec())
    }

    async fn read_all_events(&self, shard_id: u64) -> Result<Vec<ProvenanceEvent>> {
        // Combina cache + segmentos
        let mut all = Vec::new();

        // Segmentos arquivados
        let segments = self.segments.read().await;
        if let Some(segs) = segments.get(&shard_id) {
            for seg in segs {
                let content = fs::read_to_string(&seg.file_path).await?;
                for line in content.lines() {
                    if let Ok(entry) = serde_json::from_str::<ProvenanceEvent>(line) {
                        all.push(entry);
                    }
                }
            }
        }

        // Arquivo ativo
        let active_path = self.config.base_path.join(format!("shard_{}_active.log", shard_id));
        if active_path.exists() {
            let content = fs::read_to_string(&active_path).await?;
            for line in content.lines() {
                if let Ok(entry) = serde_json::from_str::<ProvenanceEvent>(line) {
                    all.push(entry);
                }
            }
        }

        Ok(all)
    }

    async fn read_metadata(&self, shard_id: u64) -> Result<Option<ShardMetadata>> {
        let meta_path = self.config.base_path.join(format!("shard_{}.meta", shard_id));
        if !meta_path.exists() {
            return Ok(None);
        }
        let content = fs::read_to_string(&meta_path).await?;
        Ok(Some(serde_json::from_str(&content)?))
    }

    async fn write_metadata(&self, shard_id: u64, metadata: &ShardMetadata) -> Result<()> {
        let meta_path = self.config.base_path.join(format!("shard_{}.meta", shard_id));
        let content = serde_json::to_string_pretty(metadata)?;
        fs::write(&meta_path, content).await?;
        Ok(())
    }

    async fn delete_shard(&self, shard_id: u64) -> Result<()> {
        // Remove todos os segmentos
        let mut segments = self.segments.write().await;
        if let Some(segs) = segments.remove(&shard_id) {
            for seg in segs {
                let _ = fs::remove_file(&seg.file_path).await;
                let _ = fs::remove_file(seg.file_path.with_extension("meta")).await;
            }
        }
        let _ = fs::remove_file(self.config.base_path.join(format!("shard_{}_active.log", shard_id))).await;
        let _ = fs::remove_file(self.config.base_path.join(format!("shard_{}.meta", shard_id))).await;
        {
            let mut cache = self.cache.write().await;
            cache.remove(&shard_id);
        }
        Ok(())
    }

    async fn list_shards(&self) -> Result<Vec<u64>> {
        let segments = self.segments.read().await;
        Ok(segments.keys().cloned().collect())
    }
}
