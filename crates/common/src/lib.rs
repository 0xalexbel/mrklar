pub mod error;
pub mod config;
pub mod merkle_proof;
pub mod proto {
    tonic::include_proto!("mrklar.v1");
}

use error::Error;
use merkle_proof::MerkleProof;
use proto::{
    download_response, upload_request, DownloadResponse, Entry, FileMetadata, ProofResponse, UploadRequest
};

// Helper
impl UploadRequest {
    pub fn new_metadata(filename: &str) -> Self {
        UploadRequest {
            r#type: Some(upload_request::Type::Metadata(FileMetadata {
                filename: filename.to_string(),
            })),
        }
    }

    pub fn new_sha256(sha256: Vec<u8>) -> Self {
        UploadRequest {
            r#type: Some(upload_request::Type::Sha256(sha256)),
        }
    }

    pub fn new_chunk(chunk: Vec<u8>) -> Self {
        UploadRequest {
            r#type: Some(upload_request::Type::Chunk(chunk)),
        }
    }

    // panics if not of type chunk
    pub fn as_mut_chunk(&mut self) -> &mut Vec<u8> {
        match self.r#type.as_mut().unwrap() {
            upload_request::Type::Chunk(c) => c,
            _ => panic!("Internal error"),
        }
    }
}

// Helper
impl DownloadResponse {
    pub fn new_entry(filename: &str, merkle_proof: MerkleProof) -> Result<Self, Error> {
        let merkle_proof_vec = merkle_proof.encode_bin()?;

        Ok(DownloadResponse {
            r#type: Some(download_response::Type::Entry(Entry {
                metadata: Some(FileMetadata {
                    filename: filename.to_string(),
                }),
                merkle_proof: merkle_proof_vec,
            })),
        })
    }

    pub fn new_chunk(chunk: Vec<u8>) -> Self {
        DownloadResponse {
            r#type: Some(download_response::Type::Chunk(chunk)),
        }
    }
}

// Helper
impl ProofResponse {
    pub fn new_proof(merkle_proof: MerkleProof) -> Result<Self, Error> {
        let merkle_proof_vec = merkle_proof.encode_bin()?;
        Ok(ProofResponse {
            merkle_proof: merkle_proof_vec
        })
    }
}
