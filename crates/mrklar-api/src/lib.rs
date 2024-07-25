use std::path::PathBuf;

use mrklar_common::merkle_proof::MerkleProof;
use mrklar_common::proto::{download_response, Empty, FileIndex, UploadRequest};
use mrklar_common::{config::NetConfig, proto::file_api_client::FileApiClient};
use mrklar_fs::{absolute_path, file_name_as_string, sha256};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::sync::mpsc;
use tokio_stream::wrappers::ReceiverStream;
use tonic::transport::Channel;
use tonic::Request;
use url::Url;

pub mod error;
use error::ApiError;

pub struct MrklarApi {
    config: NetConfig,
}

impl MrklarApi {
    pub fn new(config: NetConfig) -> Self {
        MrklarApi { config }
    }

    fn url(&self) -> Url {
        self.config.url().unwrap()
    }

    /// Attempt to create a new `FileApiClient` by connecting to a server endpoint.
    /// specified in the `config` field.
    /// Will fail if the connection is refused or the server is not running.
    async fn connect(&self) -> Result<FileApiClient<Channel>, tonic::transport::Error> {
        let url = self.url();
        FileApiClient::connect(url.to_string()).await
    }

    /// Gets the number of entries in the remote archive
    pub async fn count(&self) -> eyre::Result<u64> {
        let mut client = self.connect().await?;
        let result = client.count(Request::new(Empty {})).await?.into_inner();
        Ok(result.value)
    }

    /// Gets the merkle root of the remote archive
    pub async fn root(&self) -> eyre::Result<Vec<u8>> {
        let mut client = self.connect().await?;
        let result = client.root(Request::new(Empty {})).await?.into_inner();
        Ok(result.merkle_root)
    }

    /// Downloads the file at `index` form the remote archive.
    /// Will fail if `index` is out of bounds.
    pub async fn download(
        &self,
        index: u64,
        output_dir: Option<PathBuf>,
        output_filename: Option<String>,
        force: bool,
    ) -> Result<(PathBuf, MerkleProof, bool), ApiError> {
        let mut client = self.connect().await?;

        let mut stream = client
            .download(Request::new(FileIndex { index }))
            .await?
            .into_inner();

        let mut merkle_proof: MerkleProof = MerkleProof::default();
        let mut filename: String = String::default();

        let output_path = match output_dir {
            Some(p) => p,
            None => PathBuf::new(),
        };

        // 1- Download metadata
        while let Some(response) = stream.message().await? {
            if response.r#type.is_none() {
                continue;
            }
            match response.r#type.unwrap() {
                download_response::Type::Entry(entry) => {
                    filename = entry.metadata.unwrap_or_default().filename;
                    merkle_proof = MerkleProof::decode_bin(entry.merkle_proof)?;
                    break;
                }
                _ => {
                    return Err(ApiError::Unexpected(
                        "Invalid message type, expecting file metadata.".to_string(),
                    ));
                }
            }
        }

        let of = output_filename.unwrap_or_default();
        let path = if !of.is_empty() {
            output_path.join(of)
        } else if !filename.is_empty() {
            output_path.join(&filename)
        } else {
            return Err(ApiError::Unexpected("Missing out filename.".to_string()));
        };

        let path = absolute_path(&path)?;
        if path.is_file() && !force {
            let p = path.to_str().unwrap_or_default().to_string();
            return Err(ApiError::DownloadFileAlreadyExists(p));
        }

        let mut tokio_file = tokio::fs::File::create(&path).await?;

        let mut succeeded = true;
        while let Some(response) = stream.message().await? {
            if response.r#type.is_none() {
                continue;
            }

            match response.r#type.unwrap() {
                download_response::Type::Chunk(c) => tokio_file.write_all(&c).await?,
                _ => {
                    succeeded = false;
                    break;
                }
            }
        }

        if !succeeded {
            // close file
            drop(tokio_file);
            // remove file (no need to handle the error)
            let _res = tokio::fs::remove_file(&path).await;
            return Err(ApiError::Unexpected(
                "Invalid message type, expecting file chunk.".to_string(),
            ));
        } else {
            tokio_file.sync_all().await?;
        }

        let file_sha256 = sha256(&path)?;
        let verified = merkle_proof.verify(&file_sha256);

        // // Verify if merkle root has been provided
        // let verified = if root.is_some() {
        //     let root_v = root.unwrap();
        //     // let root_v = match hex::decode(root.unwrap()) {
        //     //     Ok(v) => v,
        //     //     Err(_) => {
        //     //         return Err(ApiError::Unexpected(
        //     //             "Invalid merkle root hash.".to_string(),
        //     //         ))
        //     //     }
        //     // };
        //     let file_sha256 = sha256(&path)?;
        //     let ok = merkle_proof.verify(&file_sha256, &root_v);
        //     Some(ok)
        // } else {
        //     None
        // };

        Ok((path, merkle_proof, verified))
    }

    /// Compute the merkle proof of file at `index` form the remote archive.
    /// Will fail if `index` is out of bounds.
    pub async fn proof(&self, index: u64) -> Result<MerkleProof, ApiError> {
        let mut client = self.connect().await?;

        let mut stream = client
            .proof(Request::new(FileIndex { index }))
            .await?
            .into_inner();

        let mut encoded_proof: Vec<u8> = vec![];
        while let Some(proof_response) = stream.message().await? {
            let mut p = proof_response.merkle_proof;
            encoded_proof.append(&mut p);
        }

        let m = MerkleProof::decode_bin(encoded_proof)?;
        Ok(m)
    }

    /// Upload file specified by `path` to remote archive.
    /// Returns the file index and the new remote merkle root
    pub async fn upload(&self, path: &PathBuf) -> Result<(u64, Vec<u8>), ApiError> {
        let (tx, rx) = mpsc::channel::<UploadRequest>(self.config.channel_size);

        if !path.is_file() {
            return Err(ApiError::UploadFileNotFound(
                path.to_str().unwrap_or_default().to_string(),
            ));
        }

        let filename = file_name_as_string(path);
        if filename.is_empty() {
            return Err(ApiError::Unexpected("Empty filename".to_string()));
        }

        let chunk_size = self.config.chunk_size;
        let file_sha256 = sha256(path)?;
        let file_path = path.clone();

        let mut client = self.connect().await?;
        //let receiver_stream = ReceiverStream::new(rx);

        let task_handle = tokio::spawn(async move {
            // 1- Send file metadata (filename)
            let request = UploadRequest::new_metadata(&filename);
            tx.send(request).await?;

            // 2- Send file sha256
            let request = UploadRequest::new_sha256(file_sha256);
            tx.send(request).await?;

            let tokio_file = tokio::fs::File::open(file_path).await?;
            let mut handle = tokio_file.take(chunk_size as u64);

            loop {
                let mut chunk = Vec::with_capacity(chunk_size);

                // read a chunk from the file
                let n = handle.read_to_end(&mut chunk).await?;

                // reset the take limit before the next chunk
                handle.set_limit(chunk_size as u64);

                // nothing left
                if n == 0 {
                    break;
                }

                // Send the file chunk to the receiver
                let request = UploadRequest::new_chunk(chunk);
                tx.send(request).await?;

                // reached the end
                if n < chunk_size {
                    break;
                }
            }

            Ok::<(), ApiError>(())
        });

        let receiver_stream = ReceiverStream::new(rx);
        let response = client.upload(receiver_stream).await?;

        let result = match task_handle.await {
            Ok(result) => result,
            Err(_) => return Err(ApiError::Unexpected("Failed to upload file".to_string())),
        };
        if result.is_err() {
            return Err(ApiError::Unexpected("Failed to upload file".to_string()));
        }

        let ur = response.into_inner();
        let file_index = match ur.index {
            Some(fi) => fi.index,
            None => {
                return Err(ApiError::Unexpected(
                    "Failed to upload file, (did not receive file index).".to_string(),
                ))
            }
        };

        Ok((file_index, ur.merkle_root))
    }
}
