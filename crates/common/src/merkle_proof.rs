use std::fmt;

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

use crate::error::Error;

pub const NULL_HASH: [u8; 32] = [0; 32];

#[derive(Debug, Serialize, Deserialize, Default, Clone)]
pub struct MerkleProofHash {
    left: bool,
    hash: Vec<u8>,
}

impl MerkleProofHash {
    pub fn new_left(hash: Vec<u8>) -> Self {
        MerkleProofHash { left: true, hash }
    }
    pub fn new_right(hash: Vec<u8>) -> Self {
        MerkleProofHash { left: false, hash }
    }
}

impl fmt::Display for MerkleProofHash {
    fn fmt(&self, fmt: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(fmt, "{} {}", (self.left as u8), hex::encode(&self.hash))?;
        Ok(())
    }
}

#[derive(Debug, Serialize, Deserialize, Default, Clone)]
pub struct MerkleProof {
    root: Vec<u8>,
    hashes: Vec<MerkleProofHash>,
}

impl fmt::Display for MerkleProof {
    fn fmt(&self, fmt: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(fmt, "Merkle proof (len={}):", self.hashes.len())?;
        if !self.hashes.is_empty() {
            for i in 0..(self.hashes.len()-1) {
                writeln!(fmt, "{}", self.hashes[i])?;
            }
            write!(fmt, "{}", self.hashes.last().unwrap())?;
        }
        Ok(())
    }
}

impl MerkleProof {
    pub fn from_raw_parts(root: Vec<u8>, hashes: Vec<MerkleProofHash>) -> Self {
        MerkleProof { 
            root,
            hashes 
        }
    }

    pub fn root(&self) -> &Vec<u8> {
        &self.root
    }

    pub fn null_hash() -> Vec<u8> {
        NULL_HASH.to_vec()
    }

    pub fn encode_bin(&self) -> Result<Vec<u8>, Error> {
        bincode::serialize(self).map_err(|_| Error::MerkleProofEncodeBin )
    }

    pub fn decode_bin(encoded: Vec<u8>) -> Result<Self, Error> {
        bincode::deserialize(&encoded[..]).map_err(|_| Error::MerkleProofDecodeBin)
    }

    pub fn sha256_pair(left: &Vec<u8>, right: &Vec<u8>) -> Vec<u8> {
        let mut hasher = Sha256::new();
        hasher.update(left);
        hasher.update(right);
        hasher.finalize().to_vec()
    }

    pub fn sha256_pair_hex(left: &str, right: &str) -> Vec<u8> {
        MerkleProof::sha256_pair(&hex::decode(left).unwrap(), &hex::decode(right).unwrap())
    }

    // pub fn _verify(&self, input: &Vec<u8>, root: &Vec<u8>) -> bool {
    //     if self.hashes.is_empty() {
    //         return false;
    //     }

    //     let mut hasher = Sha256::new();

    //     if self.hashes[0].left {
    //         hasher.update(&self.hashes[0].hash);
    //         hasher.update(input);
    //     } else {
    //         hasher.update(input);
    //         hasher.update(&self.hashes[0].hash);
    //     }

    //     for i in 1..self.hashes.len() {
    //         let h = hasher.finalize_reset();

    //         if self.hashes[i].left {
    //             hasher.update(&self.hashes[i].hash);
    //             hasher.update(h);
    //         } else {
    //             hasher.update(h);
    //             hasher.update(&self.hashes[i].hash);
    //         }
    //     }

    //     let hash = hasher.finalize().to_vec(); 
    //     let ok1 = hash == *root;
    //     let ok2 = hash == self.root;
    //     assert_eq!(ok1, ok2);
        
    //     ok1
    // }
    pub fn verify(&self, input: &Vec<u8>) -> bool {
        if self.hashes.is_empty() {
            return false;
        }

        let mut hasher = Sha256::new();

        if self.hashes[0].left {
            hasher.update(&self.hashes[0].hash);
            hasher.update(input);
        } else {
            hasher.update(input);
            hasher.update(&self.hashes[0].hash);
        }

        for i in 1..self.hashes.len() {
            let h = hasher.finalize_reset();

            if self.hashes[i].left {
                hasher.update(&self.hashes[i].hash);
                hasher.update(h);
            } else {
                hasher.update(h);
                hasher.update(&self.hashes[i].hash);
            }
        }

        hasher.finalize().to_vec() == self.root
    }
}

#[cfg(test)]
mod test {
    use super::MerkleProof;
    use sha2::{Digest, Sha256};

    #[test]
    fn test() {
        let left_hex = "edeaaff3f1774ad2888673770c6d64097e391bc362d7d6fb34982ddf0efd18cb";
        let right_hex = "1c27ae443e93ef623d8670b611ae1d7f7d71c7f103258ff8ce0c90fab557dfd8";
        let left = hex::decode(left_hex).unwrap();
        let right = hex::decode(right_hex).unwrap();
        let pair = MerkleProof::sha256_pair(&left, &right);
        assert_eq!(
            "5485e2e93b173cbe9abfce3d738ff80d444daa9b1e1717551bbd599bb2d4a78c",
            hex::encode(&pair)
        );

        let merge = [left.clone(), right.clone()].concat();
        assert_eq!(merge.len(), left.len() + right.len());
        assert_eq!(merge[0..left.len()], left);
        assert_eq!(merge[left.len()..left.len() + right.len()], right);

        let mut hasher = Sha256::new();
        hasher.update(merge);
        let merge_hash = hasher.finalize().to_vec();
        assert_eq!(merge_hash, pair);
    }
}
