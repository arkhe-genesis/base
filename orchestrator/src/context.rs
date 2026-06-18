//! Cathedral ARKHE v28.3.2 — Contexto Isolado para Subagentes
//! Gerencia a janela de contexto de cada subagente com persistência e compressão.
//! Selo: CATHEDRAL-ARKHE-v28.3.2-CONTEXT-MODULE-2026-06-17

use std::sync::Arc;
use tokio::sync::RwLock;
use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};
use tracing::{debug, warn, info};

// ============================================================================
// 1. Estrutura do Contexto
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContextEntry {
    pub id: String,
    pub content: String,
    pub timestamp: DateTime<Utc>,
    pub role: ContextRole,         // Quem inseriu a mensagem
    pub importance: f32,            // 0.0 a 1.0 (para priorização de evicção)
    pub tokens: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ContextRole {
    System,
    User,
    Agent,
    Subagent,
    Tool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContextMetadata {
    pub total_tokens: usize,
    pub entry_count: usize,
    pub oldest_entry: Option<DateTime<Utc>>,
    pub newest_entry: Option<DateTime<Utc>>,
    pub compression_ratio: f32,     // 0.0 a 1.0
}

// ============================================================================
// 2. Context — Janela de Contexto Isolada
// ============================================================================

#[derive(Debug, Clone)]
pub struct Context {
    id: String,
    entries: Arc<RwLock<Vec<ContextEntry>>>,
    max_tokens: usize,
    max_entries: usize,
    /// Estratégia de evicção: "fifo" | "lru" | "importance"
    eviction_strategy: String,
    /// Última vez que o contexto foi compactado
    last_compacted: Arc<RwLock<DateTime<Utc>>>,
}

impl Context {
    /// Cria um novo contexto isolado.
    pub fn new(id: &str) -> Self {
        Self {
            id: id.to_string(),
            entries: Arc::new(RwLock::new(Vec::new())),
            max_tokens: 4096,
            max_entries: 100,
            eviction_strategy: "importance".to_string(),
            last_compacted: Arc::new(RwLock::new(Utc::now())),
        }
    }

    pub fn with_limits(mut self, max_tokens: usize, max_entries: usize) -> Self {
        self.max_tokens = max_tokens;
        self.max_entries = max_entries;
        self
    }

    pub fn with_strategy(mut self, strategy: &str) -> Self {
        self.eviction_strategy = strategy.to_string();
        self
    }

    // ========================================================================
    // 2.1. Adição de Mensagens
    // ========================================================================

    /// Adiciona uma entrada ao contexto.
    pub async fn add(&self, content: &str, role: ContextRole, importance: f32) -> Result<(), String> {
        let tokens = Self::count_tokens(content);
        let entry = ContextEntry {
            id: uuid::Uuid::new_v4().to_string(),
            content: content.to_string(),
            timestamp: Utc::now(),
            role,
            importance: importance.clamp(0.0, 1.0),
            tokens,
        };

        let mut entries = self.entries.write().await;
        entries.push(entry);

        // Verifica limites e aplica evicção se necessário
        self.enforce_limits(&mut entries).await;

        debug!("➕ Contexto {}: adicionada entrada ({})", self.id, content.len());
        Ok(())
    }

    /// Adiciona uma mensagem do sistema.
    pub async fn add_system(&self, content: &str) -> Result<(), String> {
        self.add(content, ContextRole::System, 1.0).await
    }

    /// Adiciona uma mensagem do usuário.
    pub async fn add_user(&self, content: &str) -> Result<(), String> {
        self.add(content, ContextRole::User, 0.8).await
    }

    /// Adiciona uma mensagem de um agente.
    pub async fn add_agent(&self, content: &str) -> Result<(), String> {
        self.add(content, ContextRole::Agent, 0.7).await
    }

    /// Adiciona uma mensagem de um subagente.
    pub async fn add_subagent(&self, content: &str) -> Result<(), String> {
        self.add(content, ContextRole::Subagent, 0.6).await
    }

    // ========================================================================
    // 2.2. Consulta e Leitura
    // ========================================================================

    /// Retorna todas as entradas (para build do prompt).
    pub async fn get_entries(&self) -> Vec<ContextEntry> {
        self.entries.read().await.clone()
    }

    /// Retorna o contexto como texto (para LLM).
    pub async fn get_text(&self) -> String {
        let entries = self.entries.read().await;
        entries.iter()
            .map(|e| format!("[{}] {}: {}", format!("{:?}", e.role), e.content, e.timestamp))
            .collect::<Vec<_>>()
            .join("\n")
    }

    /// Retorna o contexto formatado como prompt.
    pub async fn get_prompt(&self) -> String {
        let entries = self.entries.read().await;
        let mut prompt = String::new();
        for entry in entries.iter() {
            let role_prefix = match entry.role {
                ContextRole::System => "Sistema",
                ContextRole::User => "Usuário",
                ContextRole::Agent => "Agente",
                ContextRole::Subagent => "Subagente",
                ContextRole::Tool => "Ferramenta",
            };
            prompt.push_str(&format!("[{}] {}\n", role_prefix, entry.content));
        }
        prompt
    }

    /// Retorna apenas as últimas N entradas.
    pub async fn get_last(&self, n: usize) -> Vec<ContextEntry> {
        let entries = self.entries.read().await;
        let start = entries.len().saturating_sub(n);
        entries[start..].to_vec()
    }

    /// Busca entradas por role.
    pub async fn get_by_role(&self, role: ContextRole) -> Vec<ContextEntry> {
        let entries = self.entries.read().await;
        entries.iter()
            .filter(|e| e.role == role)
            .cloned()
            .collect()
    }

    // ========================================================================
    // 2.3. Metadados e Estatísticas
    // ========================================================================

    /// Retorna metadados do contexto.
    pub async fn metadata(&self) -> ContextMetadata {
        let entries = self.entries.read().await;
        let total_tokens: usize = entries.iter().map(|e| e.tokens).sum();
        let entry_count = entries.len();
        let oldest = entries.first().map(|e| e.timestamp);
        let newest = entries.last().map(|e| e.timestamp);
        let compression_ratio = if entry_count > 0 {
            1.0 - (total_tokens as f32 / (entry_count * 100) as f32)
        } else {
            0.0
        };

        ContextMetadata {
            total_tokens,
            entry_count,
            oldest_entry: oldest,
            newest_entry: newest,
            compression_ratio: compression_ratio.clamp(0.0, 1.0),
        }
    }

    /// Retorna o número de entradas.
    pub async fn len(&self) -> usize {
        self.entries.read().await.len()
    }

    /// Retorna o número de tokens.
    pub async fn token_count(&self) -> usize {
        let entries = self.entries.read().await;
        entries.iter().map(|e| e.tokens).sum()
    }

    /// Verifica se o contexto está vazio.
    pub async fn is_empty(&self) -> bool {
        self.entries.read().await.is_empty()
    }

    // ========================================================================
    // 2.4. Manutenção e Evicção
    // ========================================================================

    /// Aplica limites de tamanho e número de entradas.
    async fn enforce_limits(&self, entries: &mut Vec<ContextEntry>) {
        // 1. Verifica limite de entradas
        if entries.len() > self.max_entries {
            self.evict_entries(entries, entries.len() - self.max_entries).await;
        }

        // 2. Verifica limite de tokens
        let total_tokens: usize = entries.iter().map(|e| e.tokens).sum();
        if total_tokens > self.max_tokens {
            self.compress_context(entries).await;
        }
    }

    /// Remove entradas com base na estratégia de evicção.
    async fn evict_entries(&self, entries: &mut Vec<ContextEntry>, count: usize) {
        if count == 0 { return; }

        match self.eviction_strategy.as_str() {
            "fifo" => {
                entries.drain(0..count);
            }
            "lru" => {
                // LRU: remove as mais antigas (ordem já é FIFO por timestamp)
                entries.drain(0..count);
            }
            "importance" => {
                // Remove as menos importantes
                entries.sort_by(|a, b| {
                    b.importance.partial_cmp(&a.importance).unwrap_or(std::cmp::Ordering::Equal)
                });
                let keep = entries.len().saturating_sub(count);
                entries.truncate(keep);
                entries.sort_by(|a, b| a.timestamp.cmp(&b.timestamp)); // Restaura ordem cronológica
            }
            _ => {
                entries.drain(0..count);
            }
        }
        debug!("🗑️ Contexto {}: {} entradas removidas", self.id, count);
    }

    /// Comprime o contexto removendo entradas menos importantes.
    async fn compress_context(&self, entries: &mut Vec<ContextEntry>) {
        let target_tokens = self.max_tokens * 8 / 10; // 80% do limite
        let current_tokens: usize = entries.iter().map(|e| e.tokens).sum();

        if current_tokens <= target_tokens { return; }

        // Ordena por importância
        entries.sort_by(|a, b| {
            b.importance.partial_cmp(&a.importance).unwrap_or(std::cmp::Ordering::Equal)
        });

        let mut new_entries = Vec::new();
        let mut tokens = 0;
        for entry in entries.iter() {
            if tokens + entry.tokens <= target_tokens {
                new_entries.push(entry.clone());
                tokens += entry.tokens;
            } else {
                break;
            }
        }

        // Restaura ordem cronológica
        new_entries.sort_by(|a, b| a.timestamp.cmp(&b.timestamp));
        *entries = new_entries;

        // Atualiza timestamp de compactação
        *self.last_compacted.write().await = Utc::now();

        info!("🧹 Contexto {} compactado: {} tokens", self.id, tokens);
    }

    /// Limpa todo o contexto.
    pub async fn clear(&self) {
        self.entries.write().await.clear();
        info!("🧹 Contexto {} limpo", self.id);
    }

    /// Salva o contexto em arquivo (opcional).
    pub async fn save_to_file(&self, path: &str) -> Result<(), String> {
        let entries = self.entries.read().await;
        let data = serde_json::to_string_pretty(&*entries)
            .map_err(|e| format!("Falha ao serializar: {}", e))?;
        tokio::fs::write(path, data)
            .await
            .map_err(|e| format!("Falha ao escrever: {}", e))
    }

    /// Carrega o contexto de arquivo.
    pub async fn load_from_file(&self, path: &str) -> Result<(), String> {
        let data = tokio::fs::read_to_string(path)
            .await
            .map_err(|e| format!("Falha ao ler: {}", e))?;
        let entries: Vec<ContextEntry> = serde_json::from_str(&data)
            .map_err(|e| format!("Falha ao deserializar: {}", e))?;
        *self.entries.write().await = entries;
        info!("📂 Contexto {} carregado de {}", self.id, path);
        Ok(())
    }

    // ========================================================================
    // 2.5. Utilitários
    // ========================================================================

    /// Contagem aproximada de tokens (simplificada).
    pub fn count_tokens(text: &str) -> usize {
        // Aproximação: 1 token ≈ 4 caracteres (para inglês)
        // Em produção: usar tokenizador real (tiktoken, etc.)
        (text.len() / 4).max(1)
    }

    /// Gera um resumo do contexto para logs.
    pub async fn summary(&self) -> String {
        let meta = self.metadata().await;
        format!(
            "Contexto {}: {} entradas, {} tokens, compactação: {:.2}%",
            self.id, meta.entry_count, meta.total_tokens, meta.compression_ratio * 100.0
        )
    }

    /// Obtém o ID do contexto.
    pub fn id(&self) -> &str {
        &self.id
    }
}

// ============================================================================
// 3. ContextManager — Gerencia múltiplos contextos
// ============================================================================

pub struct ContextManager {
    contexts: Arc<RwLock<Vec<Arc<Context>>>>,
}

impl ContextManager {
    pub fn new() -> Self {
        Self {
            contexts: Arc::new(RwLock::new(Vec::new())),
        }
    }

    /// Cria um novo contexto.
    pub async fn create_context(&self, id: &str) -> Arc<Context> {
        let ctx = Arc::new(Context::new(id));
        let mut contexts = self.contexts.write().await;
        contexts.push(ctx.clone());
        ctx
    }

    /// Obtém um contexto pelo ID.
    pub async fn get_context(&self, id: &str) -> Option<Arc<Context>> {
        let contexts = self.contexts.read().await;
        contexts.iter().find(|c| c.id() == id).cloned()
    }

    /// Remove um contexto.
    pub async fn remove_context(&self, id: &str) -> Result<(), String> {
        let mut contexts = self.contexts.write().await;
        let pos = contexts.iter().position(|c| c.id() == id);
        if let Some(idx) = pos {
            contexts.remove(idx);
            Ok(())
        } else {
            Err("Contexto não encontrado".to_string())
        }
    }

    /// Lista todos os contextos.
    pub async fn list_contexts(&self) -> Vec<String> {
        let contexts = self.contexts.read().await;
        contexts.iter().map(|c| c.id().to_string()).collect()
    }
}

// ============================================================================
// 4. Testes
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_context_add_and_get() {
        let ctx = Context::new("test");
        ctx.add_system("Sistema iniciado").await.unwrap();
        ctx.add_user("Olá, agente").await.unwrap();
        ctx.add_agent("Olá, usuário").await.unwrap();

        assert_eq!(ctx.len().await, 3);
        let text = ctx.get_text().await;
        assert!(text.contains("Sistema iniciado"));
    }

    #[tokio::test]
    async fn test_context_limits() {
        let ctx = Context::new("test")
            .with_limits(100, 5);

        for i in 0..10 {
            ctx.add_user(&format!("Mensagem {}", i)).await.unwrap();
        }

        assert!(ctx.len().await <= 5);
        let tokens = ctx.token_count().await;
        assert!(tokens <= 150); // 100 * 1.5 (tolerância)
    }

    #[tokio::test]
    async fn test_context_manager() {
        let manager = ContextManager::new();
        let ctx1 = manager.create_context("ctx1").await;
        let ctx2 = manager.create_context("ctx2").await;

        ctx1.add_system("Contexto 1").await.unwrap();
        ctx2.add_system("Contexto 2").await.unwrap();

        let list = manager.list_contexts().await;
        assert_eq!(list.len(), 2);
        assert!(list.contains(&"ctx1".to_string()));
    }
}
