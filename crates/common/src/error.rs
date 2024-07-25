#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Failed to serialize binary merkle proof")]
    MerkleProofEncodeBin,
    #[error("Failed to deserialize binary merkle proof")]
    MerkleProofDecodeBin,
    #[error("Invalid Url")]
    BadUrl,
}
