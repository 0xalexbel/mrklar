use mrklar_common::proto::UploadRequest;

#[derive(Debug, thiserror::Error)]
pub enum ApiError {
    #[error(transparent)]
    Io(#[from] std::io::Error),
    #[error(transparent)]
    Status(#[from] tonic::Status),
    #[error(transparent)]
    Transport(#[from] tonic::transport::Error),
    #[error(transparent)]
    SendUploadRequest(#[from] tokio::sync::mpsc::error::SendError<UploadRequest>),
    #[error("Unexpected error: {0}")]
    Unexpected(String),
    #[error("File upload: '{0}': File not found")]
    UploadFileNotFound(String),
    #[error("File download: '{0}': File already exists")]
    DownloadFileAlreadyExists(String),
    #[error(transparent)]
    Common(#[from] mrklar_common::error::Error),
}
