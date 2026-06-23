use axum::{routing::get, Router};
use std::net::SocketAddr;

#[tokio::main]
async fn main() {


    let app = Router::new()
        .route("/health", get(|| async { "OK" }));

    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
    tracing::info!("listening on {}", addr);
    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
