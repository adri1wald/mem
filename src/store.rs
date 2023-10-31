use std::env;
use std::fs::{create_dir_all, File, OpenOptions};
use std::io;
use std::path::PathBuf;

pub struct MemoryStore {
    data_file: File,
}

impl MemoryStore {
    /// Insert a new memory into the store.
    pub fn insert(&mut self, memory: &str, description: &str) -> Result<(), io::Error> {
        println!("Inserting memory: {}", memory);
        println!("Description: {}", description);
        println!("Data file: {:?}", self.data_file);
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
