#[derive(Debug, thiserror::Error)]
pub enum FsError {
    #[error(transparent)]
    Io(#[from] std::io::Error),
    #[error("{0}")]
    Unexpected(String),
}
