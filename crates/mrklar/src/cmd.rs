use crate::config::ServerConfig;
use clap::Parser;
use mrklar_common::config::{DEFAULT_SERVER_HOST_STR, DEFAULT_SERVER_PORT_STR};
use std::{net::IpAddr, path::PathBuf};

#[derive(Clone, Debug, Parser)]
pub struct ServerCmd {
    /// Port number to listen on.
    #[arg(
        long, 
        short, 
        value_name = "NUM", 
        env = "MRKLAR_PORT",
        default_value = DEFAULT_SERVER_PORT_STR, 
    )]
    pub port: u16,

    /// The hosts the server will listen on.
    #[arg(
        long,
        value_name = "IP_ADDR",
        env = "MRKLAR_IP_ADDR",
        default_value = DEFAULT_SERVER_HOST_STR
    )]
    pub host: IpAddr,

    /// Server db directory.
    #[arg(
        long, 
        value_name = "DB_DIR",
        env = "MRKLAR_DB_DIR",
    )]
    pub db_dir: PathBuf,

    /// Server files db directory.
    #[arg(
        long, 
        value_name = "FILES_DIR",
        env = "MRKLAR_FILES_DIR",
    )]
    pub files_dir: PathBuf,

    /// Enable/disable server trace [default:true].
    #[arg(
        long,
        env = "MRKLAR_TRACING",
    )]
    pub tracing: bool,

    /// Server log level.
    #[arg(
        long, 
        value_parser = ["error", "warn", "info", "debug", "trace"], 
        default_value = "info", 
        value_name = "LEVEL",
        env = "MRKLAR_TRACING_LEVEL",
    )]
    pub tracing_level: String,
}

impl ServerCmd {
    pub fn into_server_config(self) -> ServerConfig {
        ServerConfig::default()
            .with_port(self.port)
            .with_host(self.host)
            .with_db_dir(self.db_dir)
            .with_files_dir(self.files_dir)
            .with_tracing(self.tracing)
            .with_tracing_level(&self.tracing_level)
    }

    pub async fn run(self) -> eyre::Result<()> {
        let config = self.into_server_config();
        crate::try_spawn(config).await?;
        Ok(())
    }
}
