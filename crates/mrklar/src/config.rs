use mrklar_common::config::NetConfig;
use mrklar_fs::{absolute_path, create_dir_if_needed, get_test_db_dir, get_test_files_dir};
use std::{
    fmt, net::{IpAddr, SocketAddr}, path::PathBuf, str::FromStr
};

use crate::error::ServerError;

#[derive(Clone, Debug)]
pub struct ServerConfig {
    pub net: NetConfig,
    db_dir: PathBuf,
    files_dir: PathBuf,
    tracing: bool,
    tracing_level: tracing::Level,
}

impl fmt::Display for ServerConfig {
    fn fmt(&self, fmt: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(fmt, "{}", self.net)?;
        writeln!(fmt, "db_dir={:?}", self.db_dir)?;
        writeln!(fmt, "files_dir={:?}", self.files_dir)?;
        writeln!(fmt, "tracing={:?}", self.tracing)?;
        write!(fmt, "tracing_level={:?}", self.tracing_level)?;
        Ok(())
    }
}

impl ServerConfig {
    /// Sets the port to use
    #[must_use]
    pub fn with_port(mut self, port: u16) -> Self {
        self.net.port = port;
        self
    }

    /// Sets the host to use
    #[must_use]
    pub fn with_host(mut self, host: IpAddr) -> Self {
        self.net.host = host;
        self
    }

    #[must_use]
    pub fn with_db_dir(mut self, db_dir: PathBuf) -> Self {
        self.db_dir = db_dir;
        self
    }

    #[must_use]
    pub fn with_files_dir(mut self, files_dir: PathBuf) -> Self {
        self.files_dir = files_dir;
        self
    }

    pub fn with_chunk_size(mut self, chunk_size: usize) -> Self {
        self.net.chunk_size = chunk_size;
        self
    }

    pub fn with_channel_size(mut self, channel_size: usize) -> Self {
        self.net.channel_size = channel_size;
        self
    }

    pub fn with_tracing(mut self, tracing: bool) -> Self {
        self.tracing = tracing;
        self
    }

    pub fn with_tracing_level(mut self, level: &str) -> Self {
        self.tracing_level = tracing::Level::from_str(level).unwrap_or(tracing::Level::INFO);
        self
    }

    pub fn chunk_size(&self) -> usize {
        self.net.chunk_size
    }

    pub fn channel_size(&self) -> usize {
        self.net.channel_size
    }

    pub fn tracing(&self) -> bool {
        self.tracing
    }

    pub fn tracing_level(&self) -> tracing::Level {
        self.tracing_level
    }

    pub fn files_db_dir(&self) -> PathBuf {
        self.files_dir.join("db")
    }

    pub fn files_tmp_dir(&self) -> PathBuf {
        self.files_dir.join("tmp")
    }

    pub fn db_dir(&self) -> &PathBuf {
        &self.db_dir
    }

    pub fn db_file(&self) -> PathBuf {
        self.db_dir.join("db.bin")
    }

    pub fn sock_addr(&self) -> SocketAddr {
        self.net.sock_addr()
    }
}

impl Default for ServerConfig {
    fn default() -> Self {
        Self {
            net: NetConfig::default(),
            db_dir: PathBuf::default(),
            files_dir: PathBuf::default(),
            tracing: true,
            tracing_level: tracing::Level::INFO,
        }
    }
}

impl ServerConfig {
    pub fn test_default() -> Self {
        Self::default()
            .with_db_dir(get_test_db_dir().unwrap())
            .with_files_dir(get_test_files_dir().unwrap())
    }

    pub fn validate(&self) -> Result<ServerConfig, ServerError> {
        let mut config = self.clone();
        config.db_dir = absolute_path(&self.db_dir)?;
        config.files_dir = absolute_path(&self.files_dir)?;

        if !config.db_dir.is_dir() {
            return Err(ServerError::DbDirDoesNotExist(String::from(
                self.db_dir.to_str().unwrap_or(""),
            )));
        }
        if !config.files_dir.is_dir() {
            return Err(ServerError::FilesDirDoesNotExist(String::from(
                self.files_dir.to_str().unwrap_or(""),
            )));
        }

        Ok(config)
    }

    pub fn create_dirs(&self) -> Result<(), ServerError> {
        create_dir_if_needed(self.files_db_dir())?;
        create_dir_if_needed(self.files_tmp_dir())?;
        Ok(())
    }
}
