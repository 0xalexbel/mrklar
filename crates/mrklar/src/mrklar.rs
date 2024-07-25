use clap::Parser;
use mrklar::cmd::ServerCmd;

#[derive(Parser)]
#[command(name = "mrklar", version = env!("CARGO_PKG_VERSION"), next_display_order = None)]
pub struct Mrklar {
    #[command(flatten)]
    pub server: ServerCmd,
}

fn print_env_vars() {
    let env_vars = [
        "MRKLAR_IP_ADDR",
        "MRKLAR_PORT",
        "MRKLAR_DB_DIR",
        "MRKLAR_FILES_DIR",
        "MRKLAR_TRACING",
        "MRKLAR_TRACING_LEVEL",
    ];
    env_vars.iter().for_each(|e| {
        let value = std::env::var(*e).unwrap_or_default();
        println!("{}={}", *e, value);
    });
}

#[tokio::main]
async fn main() -> eyre::Result<()> {
    let app = Mrklar::parse();
    print_env_vars();
    app.server.run().await
}
