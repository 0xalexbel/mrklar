#[derive(Debug, thiserror::Error)]
pub enum MerkleTreeError {
    #[error(transparent)]
    Io(#[from] std::io::Error),
    #[error("Node hash at (level={0}, index={1}) is invalid")]
    InvalidHash(u8, usize),
    #[error("Tree is empty")]
    TreeEmpty,
    #[error("Node index {1} does not exist at level {0}")]
    NodeDoesNotExist(u8, usize),
    #[error("Too many levels in the tree")]
    TooManyLevels,
    #[error("Tree level {0} is full")]
    LevelFull(u8),
    #[error("Unexpected error")]
    UnexpectedError,
}
