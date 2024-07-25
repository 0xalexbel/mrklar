use std::io;

use crate::{error::ServerError, mem_db::MemDb, node::Node};
use mrklar_common::proto::{
    file_api_server::FileApi, upload_request, DownloadResponse, Empty, FileIndex, FileMetadata,
    ProofResponse, RootResponse, UploadRequest, UploadResponse, U64,
};
use mrklar_fs::gen_tmp_filename;
use sha2::{Digest, Sha256};
use tokio::io::AsyncReadExt;
use tokio::io::AsyncWriteExt;
use tokio::sync::mpsc;
use tokio_stream::wrappers::ReceiverStream;
use tokio_stream::StreamExt;
use tonic::{Request, Response, Status, Streaming};

#[derive(Debug)]
pub struct FileService {
    node: Node,
}

impl FileService {
    pub fn new(node: Node) -> Self {
        FileService { node }
    }
}

#[allow(dead_code)]
fn test_throw_io_error() -> Result<(), io::Error> {
    let f = std::fs::File::open("/gfdhfhfhddfgdfgd/asdfds/dfg/dfg/ggdg/dfg/sfgd/fg");
    match f {
        Ok(_) => (),
        Err(e) => {
            tracing::error!(message = "DUMMY TEST HERE!!!!");
            return Err(e);
        }
    };
    Ok(())
}

#[tonic::async_trait]
impl FileApi for FileService {
    /// Returns the number of file entries in the archive
    async fn count(&self, _: Request<Empty>) -> Result<Response<U64>, Status> {
        let file_count = self.node.file_count();
        Ok(Response::new(U64 {
            value: file_count as u64,
        }))
    }

    /// Returns the merkle root of the archive
    async fn root(&self, _: Request<Empty>) -> Result<Response<RootResponse>, Status> {
        let merkle_root = self
            .node
            .db()
            .merkle_root()
            .map_err(ServerError::MerkleTree)?;
        Ok(Response::new(RootResponse { merkle_root }))
    }

    /// Uploads a file, upon successful completion, saves the file
    /// on disk in the db directory, then computes the new merkle root.
    /// Returns the file index and the merkle root.
    async fn upload(
        &self,
        request: Request<Streaming<UploadRequest>>,
    ) -> Result<Response<UploadResponse>, Status> {
        let mut request_stream = request.into_inner();

        // create db directories if needed
        let res = self.node.config().create_dirs();
        if let Err(e) = res {
            return Err(Status::internal(e.to_string()));
        }

        let tmp_dir = self.node.config().files_tmp_dir();
        let tmp_filename = gen_tmp_filename();
        let tmp_path = tmp_dir.join(tmp_filename);
        let node = self.node.clone();

        let task_handle = tokio::spawn(async move {
            // 1- read file metadata
            let mut next = request_stream.next().await;
            let file_metadata = upload_request_file_metadata(next)?;
            let filename = &file_metadata.filename;

            if filename.is_empty() {
                return Err(ServerError::UploadInvalidFilename);
            }

            // 2- read file sha256
            next = request_stream.next().await;
            let file_sha256 = upload_request_file_sha256(next)?;
            let file_hash = file_sha256.clone();

            // Trace
            if node.config().tracing() {
                let sha256 = hex::encode(&file_sha256);
                tracing::info!(message = "upload", filename, sha256);
            }

            // 3- save file into a tmp file
            let mut tokio_file = tokio::fs::File::create(&tmp_path).await?;

            // 4- Upload bytes chunk by chunk and compute hash
            let res: Result<(), ServerError> = async move {
                let mut hasher = Sha256::new();

                loop {
                    let next = request_stream.next().await;
                    if next.is_none() {
                        break;
                    }

                    let chunk = upload_request_chunk(next)?;
                    hasher.update(&chunk);

                    tokio_file.write_all(&chunk).await?;
                }

                tokio_file.sync_all().await?;

                // Compare hash
                let hash = hasher.finalize().to_vec();
                if hash != file_hash {
                    tracing::error!(message = "upload sha256 mismatched.");
                    return Err(ServerError::UploadInvalidHash);
                }

                Ok(())
            }
            .await;

            // if task failed, remove temporary file
            // TODO: use tempfile crate instead.
            if let Err(e) = res {
                let _ = tokio::fs::remove_file(tmp_path).await;
                return Err(e);
            }

            // add_file() will do the following:
            // - move the temporary file 'tmp_path' into the db if succeeded
            // - delete the temporary file 'tmp_path' if failed internaly
            let (file_index, merkle_root) = node
                .db()
                .add_file(
                    node.config(),
                    &file_metadata.filename,
                    file_sha256,
                    &tmp_path,
                )
                .map_err(|_| {
                    ServerError::Unexpected("Unable to add file to merkle tree".to_string())
                })?;

            Ok::<(usize, Vec<u8>), ServerError>((file_index, merkle_root))
        });

        // Wait for the upload task to complete
        // retreive the task output result
        let result = match task_handle.await {
            Ok(result) => result,
            // Internal error, the JoinHandle 'task_handle' has failed to execute to completion
            Err(_) => return Err(Status::internal("Failed to upload file")),
        };

        match result {
            // upload succeded, return the file index and the new merkle root
            Ok((file_index, merkle_root)) => Ok(Response::new(UploadResponse {
                index: Some(FileIndex {
                    index: file_index as u64,
                }),
                merkle_root,
            })),
            // upload failed, forward the error to the client
            Err(e) => Err(Status::internal(e.to_string())),
        }
    }

    type ProofStream = ReceiverStream<Result<ProofResponse, Status>>;

    /// Returns the merkle proof of the file corresponding to the given index
    async fn proof(
        &self,
        request: tonic::Request<FileIndex>,
    ) -> std::result::Result<Response<Self::ProofStream>, Status> {
        let (tx, rx) =
            mpsc::channel::<Result<ProofResponse, Status>>(self.node.config().channel_size());

        let node = self.node.clone();
        let file_index = request.get_ref().index;

        tracing::info!(message = "proof", %file_index);

        tokio::spawn(async move {
            let (_, merkle_proof) =
                node.db().compute_proof_and_entry(file_index as usize)?;

            let response = ProofResponse::new_proof(merkle_proof)?;
            // will fail if rx dropped
            tx.send(Ok(response)).await?;

            Ok::<(), ServerError>(())
        });

        Ok(Response::new(ReceiverStream::new(rx)))
    }

    type DownloadStream = ReceiverStream<Result<DownloadResponse, Status>>;

    /// Downloads the file at the given index, returns its corresponding
    /// filename as well as its merkle proof.
    async fn download(
        &self,
        request: tonic::Request<FileIndex>,
    ) -> std::result::Result<Response<Self::DownloadStream>, Status> {
        let (tx, rx) =
            mpsc::channel::<Result<DownloadResponse, Status>>(self.node.config().channel_size());

        let node = self.node.clone();

        let file_index = request.get_ref().index;
        let path = MemDb::file_path_at(file_index as usize, &node.config().files_db_dir());

        tracing::info!(message = "download", %file_index);

        tokio::spawn(async move {
            // Retreive request file from the db
            let (mem_db_entry, merkle_proof) =
                node.db().compute_proof_and_entry(file_index as usize)?;

            // 1- Send file metadata (filename)
            let response = DownloadResponse::new_entry(mem_db_entry.filename(), merkle_proof)?;
            // will fail if rx dropped
            tx.send(Ok(response)).await?;

            let chunk_size = node.config().chunk_size();
            let tokio_file = tokio::fs::File::open(path).await?;
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
                let response = DownloadResponse::new_chunk(chunk);
                // will fail if rx dropped
                tx.send(Ok(response)).await?;

                // reached the end
                if n < chunk_size {
                    break;
                }
            }

            Ok::<(), ServerError>(())
        });

        Ok(Response::new(ReceiverStream::new(rx)))
    }
}

fn get_upload_request_type(
    o: Option<Result<UploadRequest, Status>>,
) -> Result<upload_request::Type, ServerError> {
    if o.is_none() {
        return Err(ServerError::EmptyMessage);
    }

    let ur = match o.unwrap() {
        Ok(ur) => ur,
        Err(e) => return Err(ServerError::Status(e)),
    };

    if ur.r#type.is_none() {
        return Err(ServerError::UndefinedMessageType);
    }

    Ok(ur.r#type.unwrap())
}

fn upload_request_file_metadata(
    o: Option<Result<UploadRequest, Status>>,
) -> Result<FileMetadata, ServerError> {
    let file_metadata = match get_upload_request_type(o)? {
        upload_request::Type::Metadata(fmd) => fmd,
        _ => return Err(ServerError::UnknownMessageType),
    };
    Ok(file_metadata)
}

fn upload_request_file_sha256(
    o: Option<Result<UploadRequest, Status>>,
) -> Result<Vec<u8>, ServerError> {
    let file_sha256 = match get_upload_request_type(o)? {
        upload_request::Type::Sha256(h) => h,
        _ => return Err(ServerError::UnknownMessageType),
    };
    Ok(file_sha256)
}

fn upload_request_chunk(o: Option<Result<UploadRequest, Status>>) -> Result<Vec<u8>, ServerError> {
    let chunk = match get_upload_request_type(o)? {
        upload_request::Type::Chunk(chunk) => chunk,
        _ => return Err(ServerError::UnknownMessageType),
    };
    Ok(chunk)
}
