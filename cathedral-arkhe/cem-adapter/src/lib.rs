use anyhow::Result;
use arkhe_wormgraph::WormGraphClient;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::{mpsc, RwLock};
use tokio_util::sync::CancellationToken;
use tracing::{error, info};
use arkhe_observer_5d::MetaGovernanceRequest;
use cathedral_atlassian::jira_client::JiraClient;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, PartialEq, Eq)]
pub enum MetaGovernanceVerdict {
    Approved,
    Rejected,
    RequiresHuman,
    RequiresCemReview,
}

#[derive(Debug, Clone)]
pub struct CemConfig {
    pub cem_agent_id: String,
    pub project_key: String,
    pub review_timeout_secs: u64,
}

impl Default for CemConfig {
    fn default() -> Self {
        Self {
            cem_agent_id: "cem-agent-1".to_string(),
            project_key: "CEM".to_string(),
            review_timeout_secs: 30,
        }
    }
}

#[derive(Debug, Clone)]
pub struct CemMetrics {
    pub alerts_processed: u64,
    pub escalated: u64,
    pub verdict: MetaGovernanceVerdict,
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub alert_id: String,
    pub received_at: chrono::DateTime<chrono::Utc>,
    pub verdict_at: chrono::DateTime<chrono::Utc>,
    pub convergence_time_ms: u64,
}

pub struct CemAdapter {
    pub config: CemConfig,
    pub jira: Option<Arc<JiraClient>>,
    _wormgraph: Arc<WormGraphClient>,
    pub alert_rx: tokio::sync::Mutex<mpsc::Receiver<MetaGovernanceRequest>>,
    pub metrics_tx: mpsc::Sender<CemMetrics>,
    _active_requests: Arc<RwLock<HashMap<String, String>>>,
    pub cancellation_token: CancellationToken,
}

impl CemAdapter {
    pub fn new(
        config: CemConfig,
        jira: Arc<JiraClient>,
        wormgraph: Arc<WormGraphClient>,
        alert_rx: mpsc::Receiver<MetaGovernanceRequest>,
        metrics_tx: mpsc::Sender<CemMetrics>,
    ) -> Self {
        Self {
            config,
            jira: Some(jira),
            _wormgraph: wormgraph,
            alert_rx: tokio::sync::Mutex::new(alert_rx),
            metrics_tx,
            _active_requests: Arc::new(RwLock::new(HashMap::new())),
            cancellation_token: CancellationToken::new(),
        }
    }

    pub fn new_mock(
        config: CemConfig,
        wormgraph: Arc<WormGraphClient>,
        alert_rx: mpsc::Receiver<MetaGovernanceRequest>,
        metrics_tx: mpsc::Sender<CemMetrics>,
    ) -> Self {
        Self {
            config,
            jira: None,
            _wormgraph: wormgraph,
            alert_rx: tokio::sync::Mutex::new(alert_rx),
            metrics_tx,
            _active_requests: Arc::new(RwLock::new(HashMap::new())),
            cancellation_token: CancellationToken::new(),
        }
    }

    pub fn new_with_mock(
        config: CemConfig,
        alert_rx: mpsc::Receiver<MetaGovernanceRequest>,
    ) -> Self {
        let (metrics_tx, _) = mpsc::channel(100);
        let wormgraph = Arc::new(WormGraphClient::new());
        Self::new_mock(config, wormgraph, alert_rx, metrics_tx)
    }

    pub async fn start(&self) -> Result<()> {
        info!("⚖️ CEM Adapter iniciado.");
        loop {
            let mut rx = self.alert_rx.lock().await;
            tokio::select! {
                Some(alert) = rx.recv() => {
                    if let Err(e) = self.process_alert(alert).await {
                        error!("Erro no processamento do alerta: {}", e);
                    }
                }
                _ = self.cancellation_token.cancelled() => {
                    info!("⚖️ CEM Adapter encerrado gracefully.");
                    break;
                }
            }
        }
        Ok(())
    }

    pub async fn process_alert(&self, alert: MetaGovernanceRequest) -> Result<()> {
        let received_at = chrono::Utc::now();
        if let Some(_jira) = &self.jira {
            // Em produção chamaria jira.create_issue
        } else {
            info!("CEM mock: processando alerta {}", alert.request_id);
        }

        tokio::time::sleep(std::time::Duration::from_millis(rand::random::<u64>() % 150 + 50)).await;

        let verdict = MetaGovernanceVerdict::Approved;
        let verdict_at = chrono::Utc::now();
        let convergence_time_ms = (verdict_at - received_at).num_milliseconds() as u64;

        let _ = self.metrics_tx.send(CemMetrics {
            alerts_processed: 1,
            escalated: 1,
            verdict: verdict.clone(),
            timestamp: chrono::Utc::now(),
            alert_id: alert.request_id.clone(),
            received_at,
            verdict_at,
            convergence_time_ms,
        }).await;

        Ok(())
    }

    pub fn shutdown(&self) {
        self.cancellation_token.cancel();
    }
}
