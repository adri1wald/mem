use serde::{Deserialize, Serialize};
use serde_json;
use std::env;
use std::fs::{create_dir_all, File, OpenOptions};
use std::io::{self, Seek, SeekFrom};
use std::path::PathBuf;

/// A memory.
#[derive(Serialize, Deserialize)]
struct Memory {
    value: String,
    description: String,
}

/// A memory database.
///
/// TODO: optimize all this for fast insertion and retrieval.
#[derive(Serialize, Deserialize)]
struct MemoryDB {
    memories: Vec<Memory>,
    embeddings: Vec<f32>,
}

/// A store for memories.
///
/// Memories have a description and a value. The description is used for semantic retrieval.
pub struct MemoryStore {
    data_file: File,
}

impl MemoryStore {
    const EMBEDDING_SIZE: usize = 512;

    /// Insert a new memory into the store.
    pub fn insert(&mut self, memory: &str, description: &str) -> Result<(), io::Error> {
        let mut db = self.load_db()?;
        db.memories.push(Memory {
            value: memory.to_string(),
            description: description.to_string(),
        });
        db.embeddings.extend(vec![0.0; Self::EMBEDDING_SIZE]);
        self.save_db(&db)?;
        Ok(())
    }

    /// Get a memory from the store.
    pub fn get(&self, description: &str) -> Result<Option<String>, io::Error> {
        println!("Getting memory: {}", description);
        println!("Data file: {:?}", self.data_file);
        Ok(None)
    }

    /// List memories from the store.
    pub fn list(&self, description: &str, count: usize) -> Result<Vec<String>, io::Error> {
        println!("Listing memories: {}", description);
        println!("Count: {}", count);
        println!("Data file: {:?}", self.data_file);
        Ok(vec![])
    }
}

/// Loading and saving the `MemoryDB` to a file.
///
/// TODO: this is all very inefficient. Need a more efficient way to store data / cache / etc.
/// TODO: But we're just prototyping for now.
impl MemoryStore {
    /// Load the `MemoryDB` from the given `File`.
    fn load_db(&mut self) -> Result<MemoryDB, io::Error> {
        // if file is empty create a new db else load the db from the file
        let db = if self.data_file.metadata()?.len() == 0 {
            MemoryDB {
                memories: vec![],
                embeddings: vec![],
            }
        } else {
            serde_json::from_reader(&self.data_file)?
        };
        Ok(db)
    }

    /// Save the `MemoryDB` to the given `File`.
    fn save_db(&mut self, db: &MemoryDB) -> Result<(), io::Error> {
        // delete the file contents
        self.data_file.set_len(0)?;
        self.data_file.seek(SeekFrom::Start(0))?;
        serde_json::to_writer(&self.data_file, db)?;
        Ok(())
    }
}

impl MemoryStore {
    const MEM_DATA_DIR_ENV_VAR: &str = "MEM_DATA_DIR";
    const DEFAULT_DATA_DIR_NAME: &str = ".mem";
    const DATA_FILE_NAME: &str = "store.json";

    /// Load the `MemoryStore` from the default data file.
    ///
    /// The default data directory is set by the `MEM_DATA_DIR` environment variable.
    /// If this variable is not set, the default data directory is `~/.mem`.
    pub fn load() -> Result<MemoryStore, io::Error> {
        let data_file_path = Self::resolve_data_file_path();
        create_dir_all(
            data_file_path
                .parent()
                .expect("the data file path always has a parent directory"),
        )?;
        let data_file = OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .open(data_file_path)?;
        Ok(Self::from_file(data_file))
    }

    /// Create a new `MemoryStore` from the given `File`.
    ///
    /// This is useful for testing.
    pub fn from_file(data_file: File) -> MemoryStore {
        MemoryStore { data_file }
    }

    fn resolve_data_file_path() -> PathBuf {
        Self::resolve_data_dir_path().join(Self::DATA_FILE_NAME)
    }

    fn resolve_data_dir_path() -> PathBuf {
        if let Ok(data_dir) = env::var(Self::MEM_DATA_DIR_ENV_VAR) {
            PathBuf::from(data_dir)
        } else {
            let home_dir = dirs::home_dir().expect(&format!(
                "Unable to determine home directory. Set the {} environment variable to override.",
                Self::MEM_DATA_DIR_ENV_VAR
            ));
            home_dir.join(Self::DEFAULT_DATA_DIR_NAME)
        }
    }
}
