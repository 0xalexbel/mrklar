use std::{
    os::unix::fs::MetadataExt,
    path::{Path, PathBuf},
    sync::Arc,
};

use mrklar_common::merkle_proof::MerkleProof;
use mrklar_fs::{self, dir_exists, file_exists};
use mrklar_tree::{error::MerkleTreeError, merkle_tree::MerkleTree};
use parking_lot::RwLock;
use serde::{Deserialize, Serialize};

use crate::{config::ServerConfig, error::ServerError};

#[derive(Debug, Default, Clone)]
pub struct MemDb {
    inner: Arc<RwLock<MemDbInner>>,
}

impl MemDb {
    pub fn num_entries(&self) -> usize {
        self.inner.read().num_entries()
    }

    pub fn merkle_root(&self) -> Result<Vec<u8>, MerkleTreeError> {
        self.inner.read().merkle_root()
    }

    pub fn compute_proof(&self, file_index: usize) -> Result<MerkleProof, MerkleTreeError> {
        self.inner.read().compute_proof(file_index)
    }

    pub(crate) fn compute_proof_and_entry(
        &self,
        file_index: usize,
    ) -> Result<(MemDbEntry, MerkleProof), ServerError> {
        self.inner.read().compute_proof_and_entry(file_index)
    }

    pub fn file_path_at(index: usize, files_db_dir: &Path) -> PathBuf {
        MemDbInner::file_path_at(index, files_db_dir)
    }

    pub fn add_file(
        &self,
        config: &ServerConfig,
        filename: &str,
        hash: Vec<u8>,
        tmp_path: &Path,
    ) -> Result<(usize, Vec<u8>), ServerError> {
        self.inner
            .write()
            .add_file(config, filename, hash, tmp_path)
    }

    pub fn try_load(config: &ServerConfig) -> eyre::Result<Self> {
        let inner = MemDbInner::try_load(config)?;
        Ok(MemDb {
            inner: Arc::new(RwLock::new(inner)),
        })
    }

    pub fn save(&self, config: &ServerConfig) -> Result<(), ServerError> {
        self.inner.read().save(config)
    }
}

#[derive(Debug, Serialize, Deserialize, Default)]
struct MemDbInner {
    // A simple one-dimensional array used to store each file metadata.
    // Pretty simple, since a file is always referred by its index.
    entries: Vec<MemDbEntry>,
    // the database merkle tree
    tree: MerkleTree,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub(crate) struct MemDbEntry {
    filename: String,
}

impl MemDbEntry {
    pub fn filename(&self) -> &str {
        &self.filename
    }
}

impl MemDbInner {
    pub fn num_entries(&self) -> usize {
        self.entries.len()
    }

    fn file_path_at(index: usize, files_db_dir: &Path) -> PathBuf {
        let mut file_path = PathBuf::new();
        file_path.push(files_db_dir);
        file_path.push(format!("{}", index));
        file_path
    }

    pub fn merkle_root(&self) -> Result<Vec<u8>, MerkleTreeError> {
        match self.tree.root_hash() {
            Ok(r) => Ok(r.clone()),
            Err(e) => Err(e),
        }
    }

    pub fn add_file(
        &mut self,
        config: &ServerConfig,
        filename: &str,
        hash: Vec<u8>,
        tmp_path: &Path,
    ) -> Result<(usize, Vec<u8>), ServerError> {
        self.tree
            .add_leaf(hash)
            .map_err(ServerError::MerkleTree)
            .and_then(|file_index| {
                // add file metadata
                self.entries.push(MemDbEntry {
                    filename: filename.to_string(),
                });
                assert!(file_index == self.entries.len() - 1);

                // compute new root (should never fail)
                let root_hash = self.tree.root_hash().unwrap().clone();

                // move file into db
                let dst_path = MemDbInner::file_path_at(file_index, &config.files_db_dir());

                // this should never fail!
                // TODO rollback if failure
                std::fs::rename(tmp_path, dst_path)?;

                self.save(config)?;

                Ok((file_index, root_hash))
            })
            .map_err(|e| {
                // in case of failure, remove tmp file
                let _ = std::fs::remove_file(tmp_path);
                e
            })
    }

    pub fn compute_proof_and_entry(
        &self,
        file_index: usize,
    ) -> Result<(MemDbEntry, MerkleProof), ServerError> {
        if file_index >= self.num_entries() {
            return Err(ServerError::FileIndexDoesNotExist(file_index));
        }
        let entry = self.entries[file_index].clone();
        let proof = self.compute_proof(file_index);
        match proof {
            Ok(proof) => Ok((entry, proof)),
            Err(e) => Err(ServerError::MerkleTree(e)),
        }
    }

    pub fn compute_proof(&self, file_index: usize) -> Result<MerkleProof, MerkleTreeError> {
        self.tree.proof_at(file_index)
    }

    pub fn try_load(config: &ServerConfig) -> Result<Self, ServerError> {
        use std::fs::File;
        use std::io::BufReader;

        if !dir_exists(config.db_dir()) {
            return Ok(MemDbInner::default());
        }

        let db_file = config.db_file();
        let db_file_str = db_file.display().to_string();

        if !file_exists(&db_file) {
            tracing::info!("db file does not exist (path={:?})", db_file_str);
            return Ok(MemDbInner::default());
        }

        let file = File::open(&db_file)?;
        let db_size_in_bytes = file.metadata().map(|m| m.size()).unwrap_or(0);
        let reader = BufReader::new(file);

        let db: MemDbInner = bincode::deserialize_from(reader).map_err(|_| ServerError::DbLoad)?;

        if config.tracing() {
            tracing::info!(
                "load db file (path={:?}, size={} bytes)",
                db_file_str,
                db_size_in_bytes
            );
        }

        // Todo:
        // - check db integrity ?
        // - verify db.entries.len() == db.tree.leaf_count()

        Ok(db)
    }

    pub fn save(&self, config: &ServerConfig) -> Result<(), ServerError> {
        use std::fs::{self, File};
        use std::io::BufWriter;

        let db_dir = config.db_dir();
        if !dir_exists(db_dir) {
            fs::create_dir(db_dir)?;
        }

        let db_file = config.db_file();

        let file = File::create(db_file)?;
        let mut writer = BufWriter::new(file);

        bincode::serialize_into(&mut writer, self).map_err(|_| ServerError::DbSave)
    }
}
