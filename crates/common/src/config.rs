use std::{fmt, net::{IpAddr, Ipv4Addr, SocketAddr}};

use url::Url;

use crate::error::Error;

pub const DEFAULT_SERVER_PORT: u16 = 10000;
pub const DEFAULT_SERVER_PORT_STR: &str = "10000";
pub const DEFAULT_SERVER_HOST_STR: &str = "127.0.0.1";
pub const DEFAULT_SERVER_URL_STR: &str = "http://127.0.0.1:10000";

pub const DEFAULT_CHANNEL_SIZE: usize = 4;
pub const DEFAULT_CHUNK_SIZE: usize = 1024 * 1024;

#[derive(Clone, Debug)]
pub struct NetConfig {
    pub port: u16,
    pub host: IpAddr,
    pub chunk_size: usize,
    pub channel_size: usize,
}

impl Default for NetConfig {
    fn default() -> Self {
        Self {
            port: DEFAULT_SERVER_PORT,
            host: IpAddr::V4(Ipv4Addr::LOCALHOST),
            chunk_size: DEFAULT_CHUNK_SIZE,
            channel_size: DEFAULT_CHANNEL_SIZE,
        }
    }
}

impl fmt::Display for NetConfig {
    fn fmt(&self, fmt: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(fmt, "port={}", self.port)?;
        writeln!(fmt, "host={:?}", self.host)?;
        writeln!(fmt, "chunk_size={:?}", self.chunk_size)?;
        write!(fmt, "channel_size={:?}", self.channel_size)?;
        Ok(())
    }
}

impl NetConfig {
    /// Sets the port to use
    #[must_use]
    pub fn with_port(mut self, port: u16) -> Self {
        self.port = port;
        self
    }

    /// Sets the host to use
    #[must_use]
    pub fn with_host(mut self, host: IpAddr) -> Self {
        self.host = host;
        self
    }

    pub fn sock_addr(&self) -> SocketAddr {
        SocketAddr::new(self.host, self.port)
    }

    pub fn url(&self) -> Result<Url, Error> {
        let mut url = Url::parse(DEFAULT_SERVER_URL_STR).map_err(|_| Error::BadUrl)?;
        url.set_ip_host(self.host).map_err(|_| Error::BadUrl)?;
        url.set_port(Some(self.port)).map_err(|_| Error::BadUrl)?;
        Ok(url)
    }
}
