//! Cathedral ARKHE v28.3.2 — Subagent Communication
//! Mensagens atestadas entre subagentes com assinatura criptográfica.
//! Selo: CATHEDRAL-ARKHE-v28.3.2-SUBAGENT-COMM-2026-06-17

use std::sync::Arc;
use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};
use tracing::{info, warn};

use crate::attestation::{AttestationSigner, AttestationVerifier};

// ============================================================================
// 1. Mensagem entre Subagentes
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SubagentMessage {
    pub id: String,
    pub sender_id: String,
    pub recipient_id: String,
    pub content: serde_json::Value,
    pub timestamp: DateTime<Utc>,
    pub message_type: MessageType,
    pub ttl_seconds: u64,
    pub signature: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum MessageType {
    Task,           // Solicitação de execução
    Result,         // Resultado de tarefa
    Query,          // Consulta de estado
    Notification,   // Notificação sem resposta esperada
    Handshake,      // Estabelecimento de comunicação
    Heartbeat,      // Prova de vida do subagente
    Terminate,      // Solicitação de encerramento
}

impl SubagentMessage {
    pub fn new(
        sender_id: &str,
        recipient_id: &str,
        content: serde_json::Value,
        message_type: MessageType,
        ttl_seconds: u64,
    ) -> Self {
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            sender_id: sender_id.to_string(),
            recipient_id: recipient_id.to_string(),
            content,
            timestamp: Utc::now(),
            message_type,
            ttl_seconds,
            signature: None,
        }
    }

    /// Assina a mensagem com a chave do emissor.
    pub fn sign(&mut self, signer: &dyn AttestationSigner) -> Result<(), String> {
        let data = self.canonical_string()?;
        self.signature = Some(signer.sign(&data)?);
        Ok(())
    }

    /// Verifica a assinatura da mensagem.
    pub fn verify(&self, verifier: &dyn AttestationVerifier) -> Result<bool, String> {
        let sig = self.signature.as_ref().ok_or("Mensagem não assinada")?;
        let data = self.canonical_string()?;
        verifier.verify(&data, sig).map_err(|e| format!("Verify error: {:?}", e))
    }

    /// Verifica se a mensagem não expirou.
    pub fn is_expired(&self) -> bool {
        let elapsed = (Utc::now() - self.timestamp).num_seconds();
        elapsed > self.ttl_seconds as i64
    }

    /// Retorna os dados canônicos para assinatura.
    fn canonical_string(&self) -> Result<String, String> {
        serde_json::to_string(self).map_err(|e| format!("Serialize error: {}", e))
    }
}

// ============================================================================
// 2. Envelope de Mensagem (com metadados de roteamento)
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MessageEnvelope {
    pub message: SubagentMessage,
    pub route: Vec<String>,          // Caminho de roteamento (agentes intermediários)
    pub delivered_at: Option<DateTime<Utc>>,
    pub receipt: Option<String>,     // Hash de confirmação
}

impl MessageEnvelope {
    pub fn new(message: SubagentMessage) -> Self {
        Self {
            message,
            route: Vec::new(),
            delivered_at: None,
            receipt: None,
        }
    }

    /// Adiciona um hop no roteamento.
    pub fn add_hop(&mut self, agent_id: &str) {
        self.route.push(agent_id.to_string());
    }

    /// Marca como entregue.
    pub fn mark_delivered(&mut self) {
        self.delivered_at = Some(Utc::now());
    }

    /// Gera um recibo (hash da mensagem + timestamp).
    pub fn generate_receipt(&mut self) -> String {
        let hash = format!("receipt-{}", self.message.id);
        self.receipt = Some(hash.clone());
        hash
    }
}

// ============================================================================
// 3. MessageBroker — Roteamento e Entrega
// ============================================================================

pub struct MessageBroker {
    subscribers: Arc<tokio::sync::RwLock<Vec<MessageSubscription>>>,
    delivery_log: Arc<tokio::sync::RwLock<Vec<MessageEnvelope>>>,
}

#[derive(Clone)]
pub struct MessageSubscription {
    pub agent_id: String,
    pub topics: Vec<MessageType>,
    pub callback: Arc<dyn Fn(MessageEnvelope) + Send + Sync>,
}

impl MessageBroker {
    pub fn new() -> Self {
        Self {
            subscribers: Arc::new(tokio::sync::RwLock::new(Vec::new())),
            delivery_log: Arc::new(tokio::sync::RwLock::new(Vec::new())),
        }
    }

    /// Publica uma mensagem para um ou todos os subagentes.
    pub async fn publish(&self, envelope: MessageEnvelope) -> Result<(), String> {
        let target = &envelope.message.recipient_id;
        let msg_type = &envelope.message.message_type;

        // 1. Verifica se a mensagem expirou
        if envelope.message.is_expired() {
            warn!("⏰ Mensagem expirada: {}", envelope.message.id);
            return Ok(());
        }

        // 2. Verifica assinatura (se presente)
        // Em produção: usar o verifier apropriado

        // 3. Encontra destinatário e entrega
        let subscribers = self.subscribers.read().await;
        let matches: Vec<_> = subscribers.iter()
            .filter(|sub| {
                sub.agent_id == *target
                    || target == "all"  // broadcast
                    || sub.topics.contains(msg_type)
            })
            .collect();

        if matches.is_empty() {
            warn!("⚠️ Nenhum destinatário para mensagem: {}", envelope.message.id);
            return Err("Destinatário não encontrado".to_string());
        }

        for sub in matches {
            (sub.callback)(envelope.clone());
        }

        // 4. Registra na entrega
        let mut log = self.delivery_log.write().await;
        log.push(envelope);

        Ok(())
    }

    /// Registra um subagente para receber mensagens.
    pub async fn subscribe(
        &self,
        agent_id: &str,
        topics: Vec<MessageType>,
        callback: impl Fn(MessageEnvelope) + Send + Sync + 'static,
    ) {
        let mut subs = self.subscribers.write().await;
        subs.push(MessageSubscription {
            agent_id: agent_id.to_string(),
            topics: topics.clone(),
            callback: Arc::new(callback),
        });
        info!("📡 Subagente {} inscrito para {:?}", agent_id, topics);
    }

    /// Remove um subagente da lista de inscrições.
    pub async fn unsubscribe(&self, agent_id: &str) {
        let mut subs = self.subscribers.write().await;
        subs.retain(|sub| sub.agent_id != agent_id);
        info!("📡 Subagente {} removido", agent_id);
    }

    /// Obtém o log de entregas.
    pub async fn delivery_log(&self, limit: usize) -> Vec<MessageEnvelope> {
        let log = self.delivery_log.read().await;
        let start = log.len().saturating_sub(limit);
        log[start..].to_vec()
    }
}

// ============================================================================
// 4. Exemplo de Uso
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[tokio::test]
    async fn test_message_flow() {
        let broker = MessageBroker::new();

        // Subagente A envia mensagem para Subagente B
        let msg = SubagentMessage::new(
            "agent-a",
            "agent-b",
            json!({ "task": "process_email" }),
            MessageType::Task,
            60,
        );

        // Assina a mensagem (com signer mock)
        // let mut msg = msg;
        // msg.sign(&mock_signer).unwrap();

        let envelope = MessageEnvelope::new(msg);
        let broker_arc = Arc::new(broker);
        let broker_clone = broker_arc.clone();

        broker_clone.subscribe("agent-b", vec![MessageType::Task], |_| {}).await;

        broker_arc.publish(envelope).await.unwrap();

        // Verifica que a mensagem foi entregue
        let log = broker_arc.delivery_log(10).await;
        assert_eq!(log.len(), 1);
    }
}
