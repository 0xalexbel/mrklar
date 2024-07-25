use mrklar::ServerConfig;

pub async fn start_server(config: ServerConfig) {
    tokio::spawn(async move { mrklar::spawn(config).await });
}

#[tokio::test(flavor = "multi_thread")]
async fn test_spawn() {
    let config = ServerConfig::test_default().with_tracing(false);

    start_server(config.clone()).await;

    tokio::time::sleep(std::time::Duration::from_millis(700)).await;
}
