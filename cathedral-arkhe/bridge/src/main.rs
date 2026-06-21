//! bridge/src/main.rs — Servidor gRPC completo com CLI e graceful shutdown
//! Selo: CATHEDRAL-ARKHE-BRIDGE-SERVER-v1.0.0

use std::sync::Arc;
use clap::Parser;
use tokio::sync::{mpsc, RwLock};
use tokio_util::sync::CancellationToken;
use tracing_subscriber;
use tracing::info;
use tonic::codegen::CompressionEncoding;

mod grpc_service;
mod tree_validator;
mod wormgraph_client;
mod governance_hook;
mod ethical_filter;
mod health;
mod metrics;

use grpc_service::CathedralGrpcService;
use tree_validator::TreeManager;
use wormgraph_client::WormGraphClient;
use governance_hook::{HierarchicalEthicalGuardian, HierarchicalGovernanceConfig};

pub mod cathedral {
    pub mod v1 {
        tonic::include_proto!("cathedral.v1");
        pub const FILE_DESCRIPTOR_SET: &[u8] = tonic::include_file_descriptor_set!("cathedral_descriptor");
    }
}

// ============================================================
// CLI
// ============================================================

#[derive(Parser)]
#[command(name = "cathedral-bridge", about = "Cathedral ARKHE Bridge Server")]
struct Args {
    #[arg(short, long, default_value = "0.0.0.0:9002")]
    addr: String,

    #[arg(long, default_value = "./wormgraph_data")]
    data_dir: String,

    #[arg(long, default_value = "default-tree")]
    default_tree: String,

    #[arg(long, default_value = "coordinator-1")]
    root_agent: String,

    #[arg(long, default_value = "coordinator")]
    root_role: String,

    #[arg(long)]
    jira_endpoint: Option<String>,

    #[arg(long)]
    jira_token: Option<String>,

    #[arg(long)]
    cem_project_key: Option<String>,

    #[arg(short, long, default_value = "info")]
    log_level: String,
}

// ============================================================
// MAIN
// ============================================================

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let args = Args::parse();

    // Configura logging
    tracing_subscriber::fmt()
        .with_env_filter(&args.log_level)
        .try_init().ok();

    info!("🏛️ Cathedral Bridge v{}", env!("CARGO_PKG_VERSION"));
    info!("   Listening on: {}", args.addr);
    info!("   Data dir: {}", args.data_dir);
    info!("   Tree: {}", args.default_tree);

    // ============================================================
    // 1. INICIALIZA COMPONENTES
    // ============================================================

    // Tree Manager
    let tree_manager = Arc::new(RwLock::new(TreeManager::new()));
    {
        let mut tm = tree_manager.write().await;
        tm.register_tree(&args.default_tree, &args.root_agent, &args.root_role)?;
        info!("🌳 Árvore '{}' registrada", args.default_tree);
    }

    // WormGraph Client (com storage persistente)
    use arkhe_wormgraph::storage_file::{HardenedFileStorage, FileStorageConfig};
    let storage = Arc::new(
        HardenedFileStorage::new(FileStorageConfig {
            base_path: std::path::PathBuf::from(&args.data_dir),
            enable_compaction: true,
            enable_retention: true,
            ..Default::default()
        }).await?
    );
    let wormgraph = Arc::new(WormGraphClient::new_with_storage(storage));
    info!("🗄️ WormGraph storage inicializado");

    // Canal para governança
    let (tx, _) = mpsc::channel(100);

    // Guardian
    let guardian = Arc::new(HierarchicalEthicalGuardian::new(
        HierarchicalGovernanceConfig::default(),
        tree_manager.clone(),
        tx,
    ));

    // Serviço gRPC
    let service = CathedralGrpcService::new(
        tree_manager.clone(),
        wormgraph.clone(),
        guardian,
    );

    // ============================================================
    // 2. GRACEFUL SHUTDOWN
    // ============================================================

    let cancellation_token = CancellationToken::new();
    let ctrl_c_token = cancellation_token.clone();

    // Captura Ctrl+C
    tokio::spawn(async move {
        tokio::signal::ctrl_c().await.ok();
        info!("🛑 Recebido Ctrl+C, iniciando graceful shutdown...");
        ctrl_c_token.cancel();
    });

    // ============================================================
    // 3. HTTP HEALTHCHECK & METRICS
    // ============================================================
    let http_addr: std::net::SocketAddr = "0.0.0.0:8080".parse()?;

    let app = axum::Router::new()
        .merge(health::health_router())
        .merge(metrics::metrics_router());

    let http_server = tokio::spawn(async move {
        let listener = tokio::net::TcpListener::bind(&http_addr).await.unwrap();
        axum::serve(listener, app).await.unwrap();
    });

    // ============================================================
    // 4. SERVIDOR gRPC
    // ============================================================

    let addr = args.addr.parse()?;

    info!("🚀 Servidor gRPC iniciado em {}", addr);

    // Inclui reflection para debugging
    let reflection_service = tonic_reflection::server::Builder::configure()
        .register_encoded_file_descriptor_set(crate::cathedral::v1::FILE_DESCRIPTOR_SET)
        .build()?;

    let grpc_server = tonic::transport::Server::builder()
        .accept_compressed(CompressionEncoding::Gzip)
        .accept_compressed(CompressionEncoding::Zstd)
        .add_service(cathedral::v1::cathedral_bridge_server::CathedralBridgeServer::new(service))
        .add_service(reflection_service)
        .serve_with_shutdown(addr, async {
            cancellation_token.cancelled().await;
            info!("🛑 Servidor encerrando...");
        });

    tokio::select! {
        _ = grpc_server => info!("gRPC servidor encerrado"),
        _ = http_server => info!("HTTP healthcheck encerrado"),
    }

    info!("👋 Servidor encerrado");
    Ok(())
}
