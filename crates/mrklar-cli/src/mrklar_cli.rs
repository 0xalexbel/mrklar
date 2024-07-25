use std::{net::IpAddr, path::{Path, PathBuf}, str::FromStr};

use clap::{Parser, Subcommand};
use mrklar_common::config::{NetConfig, DEFAULT_SERVER_HOST_STR, DEFAULT_SERVER_PORT_STR};
use mrklar_api::MrklarApi;

#[derive(Parser)]
#[command(name = "mrklar-cli", version = env!("CARGO_PKG_VERSION"), next_display_order = None)]
pub struct Cli {
    #[command(flatten)]
    pub net: NetCmd,

    #[command(subcommand)]
    pub cmd: CliSubcommand,
}

#[derive(Clone, Debug, Parser)]
pub struct NetCmd {
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
}

impl NetCmd {
    pub fn into_net_config(self) -> NetConfig {
        NetConfig::default()
            .with_port(self.port)
            .with_host(self.host)
    }
}

#[derive(Subcommand)]
pub enum CliSubcommand {
    /// Print the number of files in the archive
    Count,
    /// Print the archive merkle root
    Root,
    /// Upload file to the remote archive
    #[command(name = "upload")]
    Upload(UploadCmd),
    /// Download file at specified index from the remote archive
    #[command(name = "download")]
    Download(DownloadCmd),
    /// Print file proof 
    #[command(name = "proof")]
    Proof(ProofCmd),
}

#[derive(Parser)]
pub struct UploadCmd {
    path: String
}

#[derive(Parser)]
pub struct DownloadCmd {
    /// File index to download
    #[arg(value_name = "INDEX")]
    index: u64,

    // /// Perform file verification using the remote archive merkle root
    // #[arg(
    //     long, 
    //     value_name = "PROOF", 
    // )]
    // pub verify: Option<String>,

    /// Directory where the downloaded file should be saved
    #[arg(
        long, 
        value_name = "DIR", 
    )]
    pub out_dir: Option<PathBuf>,

    /// Specify the filename of downloaded file
    #[arg(
        long, 
        value_name = "NAME", 
    )]
    pub out_filename: Option<String>,

    /// Override any existing file
    #[arg(
        long, 
        short,
    )]
    pub force: bool,
}

#[derive(Parser)]
pub struct ProofCmd {
    /// File index 
    #[arg(value_name = "INDEX")]
    index: u64
}

async fn run_count_cmd(api: MrklarApi) -> eyre::Result<()> {
    let result = api.count().await?;
    println!("{}", result);
    Ok(())
}

async fn run_root_cmd(api: MrklarApi) -> eyre::Result<()> {
    let result = api.root().await?;
    let root_hex = hex::encode(result);
    println!("{}", root_hex);
    Ok(())
}

async fn run_upload_cmd(api: MrklarApi, path: &Path) -> eyre::Result<()> {
    let path_buf = path.to_path_buf();
    let result = api.upload(&path_buf).await?;
    let file_index = result.0;
    let root_hex = hex::encode(result.1);
    println!("{} {}", file_index, root_hex);
    Ok(())
}

async fn run_download_cmd(api: MrklarApi, index: u64, out_dir: Option<PathBuf>, out_filename: Option<String>, force: bool) -> eyre::Result<()> {
    // let root_v = if let Some(root_hex) = root {
    //     Some(hex::decode(root_hex).map_err(|_| eyre!("Invalid merkle root hash."))?)
    // } else {
    //     None
    // };

    let result = api.download(index, out_dir, out_filename, force).await?;
    println!("path: {}", result.0.display());
    println!("{}", result.1);
    println!("verification: {}", if result.2 { "OK" } else { "FAILED" } );
    // if result.2.is_some() {
    //     let ok = result.2.unwrap();
    //     println!("verification: {}", if ok { "OK" } else { "FAILED" } );
    // }
    Ok(())
}

async fn run_proof_cmd(api: MrklarApi, index: u64) -> eyre::Result<()> {
    let result = api.proof(index).await?;
    println!("{}", result);
    Ok(())
}

#[tokio::main]
async fn main() -> eyre::Result<()> {
    let cli = Cli::parse();
    let config = cli.net.into_net_config();
    let api = MrklarApi::new(config);
    match cli.cmd {
        CliSubcommand::Count => {
            run_count_cmd(api).await?
        },
        CliSubcommand::Root => {
            run_root_cmd(api).await?
        },
        CliSubcommand::Upload(upload_cmd) => {
            let p = PathBuf::from_str(&upload_cmd.path)?;
            run_upload_cmd(api, &p).await?
        },
        CliSubcommand::Download(download_cmd) => {
            run_download_cmd(api, download_cmd.index, download_cmd.out_dir, download_cmd.out_filename, download_cmd.force).await?
        },
        CliSubcommand::Proof(proof_cmd) => {
            run_proof_cmd(api, proof_cmd.index).await?
        },
    };

    Ok(())
}
