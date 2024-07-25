use mrklar_common::proto::{DownloadResponse, ProofResponse};
use tonic::Status;

#[derive(Debug, thiserror::Error)]
pub enum ServerError {
    #[error(transparent)]
    Io(#[from] std::io::Error),
    #[error(transparent)]
    Status(#[from] tonic::Status),
    #[error("Server db directory '{0}' does not exist")]
    DbDirDoesNotExist(String),
    #[error("Server files directory '{0}' does not exist")]
    FilesDirDoesNotExist(String),
    #[error("Unexpected error: {0}")]
    Unexpected(String),
    #[error("Undefined message type")]
    UndefinedMessageType,
    #[error("Unknown message type")]
    UnknownMessageType,
    #[error("Empty message")]
    EmptyMessage,
    #[error("Upload failed, invalid hash value")]
    UploadInvalidHash,
    #[error("Upload failed, invalid filename")]
    UploadInvalidFilename,
    #[error("File index {0} does not exist")]
    FileIndexDoesNotExist(usize),
    #[error(transparent)]
    MerkleTree(#[from] mrklar_tree::error::MerkleTreeError),
    #[error("Memory DB save failed.")]
    DbSave,
    #[error("Memory DB load failed.")]
    DbLoad,
    // receiver dropped
    #[error(transparent)]
    SendDownloadResponse(
        #[from] tokio::sync::mpsc::error::SendError<Result<DownloadResponse, Status>>,
    ),
    // receiver dropped
    #[error(transparent)]
    SendProofResponse(
        #[from] tokio::sync::mpsc::error::SendError<Result<ProofResponse, Status>>,
    ),
    #[error(transparent)]
    Common(#[from] mrklar_common::error::Error),
}

impl From<ServerError> for Status {
    fn from(value: ServerError) -> Self {
        match value {
            ServerError::Io(e) => Status::from_error(Box::new(e)),
            ServerError::Status(s) => s,
            ServerError::DbDirDoesNotExist(m) => Status::not_found(m),
            ServerError::FilesDirDoesNotExist(m) => Status::not_found(m),
            ServerError::Unexpected(m) => Status::internal(m),
            ServerError::UndefinedMessageType => Status::internal(value.to_string()),
            ServerError::UnknownMessageType => Status::internal(value.to_string()),
            ServerError::EmptyMessage => Status::internal(value.to_string()),
            ServerError::UploadInvalidHash => Status::invalid_argument(value.to_string()),
            ServerError::UploadInvalidFilename => Status::invalid_argument(value.to_string()),
            ServerError::FileIndexDoesNotExist(_) => Status::not_found(value.to_string()),
            ServerError::MerkleTree(e) => Status::internal(e.to_string()),
            ServerError::SendDownloadResponse(e) => Status::internal(e.to_string()),
            ServerError::SendProofResponse(e) => Status::internal(e.to_string()),
            ServerError::Common(e) => Status::internal(e.to_string()),
            ServerError::DbSave => Status::internal(value.to_string()),
            ServerError::DbLoad => Status::internal(value.to_string()),
        }
    }
}
