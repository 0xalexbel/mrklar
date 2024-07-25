use file_service::FileService;
use mem_db::MemDb;
use mrklar_common::proto::file_api_server::FileApiServer;
use node::Node;
use tonic::transport::Server;

pub mod cmd;
pub(crate) mod file_service;
pub mod mem_db;
pub(crate) mod node;

mod config;
pub use config::ServerConfig;
pub mod error;

pub async fn spawn(config: ServerConfig) {
    try_spawn(config).await.expect("failed to spawn server")
}

#[tracing::instrument]
pub async fn on_shutdown() {
    // TODO implement graceful shutdown
    tokio::signal::ctrl_c().await.ok();
    tracing::info!(message = "Shutting down server...");
}

pub async fn try_spawn(config: ServerConfig) -> eyre::Result<()> {
    let config = config.validate()?;

    if config.tracing() {
        tracing_subscriber::fmt()
            .with_max_level(config.tracing_level())
            .init();
    }

    let sock_addr = config.sock_addr();

    tracing::info!(message = "Starting server", %sock_addr);
    tracing::info!(message = "Config", %config);

    let db = MemDb::try_load(&config)?;
    let node = Node::new(config, db);

    let service = FileService::new(node);
    let svc = FileApiServer::new(service);

    Server::builder()
        .trace_fn(|_| tracing::info_span!("mrklar_server"))
        .add_service(svc)
        .serve_with_shutdown(sock_addr, on_shutdown())
        .await?;

    tracing::info!(message = "Server shutdown.", %sock_addr);

    Ok(())
}
