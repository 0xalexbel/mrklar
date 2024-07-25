use crate::{config::ServerConfig, mem_db::MemDb};

#[derive(Debug, Clone)]
pub struct Node {
    config: ServerConfig,
    db: MemDb,
}

impl Node {
    pub fn new(config: ServerConfig, db: MemDb) -> Self {
        Node { config, db }
    }

    pub fn config(&self) -> &ServerConfig {
        &self.config
    }

    pub fn db(&self) -> &MemDb {
        &self.db
    }

    pub fn file_count(&self) -> usize {
        self.db.num_entries()
    }
}
