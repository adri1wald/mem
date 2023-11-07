use anyhow::{Context, Result};
use ndarray::{Array1, Array2, ArrayView};
use openai_api_rs::v1::api as openai;
use openai_api_rs::v1::embedding::EmbeddingRequest;
use serde::{Deserialize, Serialize};
use serde_json;
use std::env;
use std::fs::{File, OpenOptions};
use std::io::{Seek, SeekFrom};
use std::path::PathBuf;

/// A memory.
#[derive(Clone, Serialize, Deserialize)]
struct Memory {
    value: String,
    description: String,
}

impl Memory {
    fn into_scored(self, score: f32) -> ScoredMemory {
        ScoredMemory {
            value: self.value,
            description: self.description,
            score,
        }
    }
}

pub struct ScoredMemory {
    pub value: String,
    pub description: String,
    pub score: f32,
}

type Embedding = Array1<f32>;
type EmbeddingMatrix = Array2<f32>;

/// A memory database.
///
/// TODO: optimize all this for fast insertion and retrieval.
#[derive(Serialize, Deserialize)]
struct MemoryDB {
    memories: Vec<Memory>,
    embeddings: EmbeddingMatrix,
}

/// A store for memories.
///
/// Memories have a description and a value. The description is used for semantic retrieval.
pub struct MemoryStore {
    data_file: File,
    openai: openai::Client,
}

impl MemoryStore {
    const EMBEDDING_SIZE: usize = 1536;
    const EMBEDDING_MODEL: &'static str = "text-embedding-ada-002";

    /// Insert a new memory into the store.
    pub fn insert(&mut self, memory: &str, description: &str) -> Result<()> {
        let mut db = self
            .load_db()
            .context("Failed to load database from file.")?;
        let embedding = self
            .embed(description)
            .context("Failed to get memory description embedding.")?;
        db.embeddings
            .push_row(ArrayView::from(&embedding))
            .expect("dimension mismatch");
        db.memories.push(Memory {
            value: memory.to_string(),
            description: description.to_string(),
        });
        self.save_db(&db)
            .context("Failed to save database to file.")?;
        Ok(())
    }

    /// Get a memory from the store.
    pub fn get(&self, description: &str) -> Result<Option<ScoredMemory>> {
        let db = self
            .load_db()
            .context("Failed to load database from file.")?;
        if db.memories.is_empty() {
            return Ok(None);
        }
        let query_embedding: Embedding = self
            .embed(description)
            .context("Failed to get query embedding.")?
            .into();
        let dot_products = db.embeddings.dot(&query_embedding);
        // get the index of the max dot product
        let max_index = dot_products
            .iter()
            .enumerate()
            .max_by(|(_, a), (_, b)| {
                a.partial_cmp(b)
                    .expect("there are no NaN values in the dot product array")
            })
            .map(|(i, _)| i)
            .unwrap();
        let memory = db.memories[max_index].clone();
        let score = dot_products[max_index];
        Ok(Some(memory.into_scored(score)))
    }

    /// List memories from the store.
    pub fn list(&self, description: &str, count: usize) -> Result<Vec<ScoredMemory>> {
        let db = self
            .load_db()
            .context("Failed to load database from file.")?;
        if db.memories.is_empty() {
            return Ok(vec![]);
        }
        let query_embedding: Embedding = self
            .embed(description)
            .context("Failed to get query embedding.")?
            .into();
        let dot_products = db.embeddings.dot(&query_embedding);
        let mut score_index_pairs: Vec<_> = dot_products
            .into_iter()
            .enumerate()
            .map(|(i, score)| (score, i))
            .collect();
        score_index_pairs.sort_by(|(a, _), (b, _)| b.partial_cmp(a).unwrap());
        score_index_pairs.truncate(count);
        let scored_memories = score_index_pairs
            .into_iter()
            .map(|(score, i)| db.memories[i].clone().into_scored(score))
            .collect();
        Ok(scored_memories)
    }

    /// Embed text using the OpenAI API.
    fn embed(&self, text: &str) -> Result<Vec<f32>> {
        let req = EmbeddingRequest::new(Self::EMBEDDING_MODEL.to_owned(), text.to_owned());
        let mut res = self
            .openai
            .embedding(req)
            .context("Failed to get embedding from OpenAI API.")?;
        if res.data[0].embedding.len() != Self::EMBEDDING_SIZE {
            return Err(anyhow::anyhow!(
                "Embedding size is not correct. Expected: {}, Got: {}",
                Self::EMBEDDING_SIZE,
                res.data[0].embedding.len()
            ));
        }
        Ok(res.data.remove(0).embedding)
    }
}

/// Loading and saving the `MemoryDB` to a file.
///
/// TODO: this is all very inefficient. Need a more efficient way to store data / cache / etc.
/// TODO: But we're just prototyping for now.
impl MemoryStore {
    /// Load the `MemoryDB` from the given `File`.
    fn load_db(&self) -> Result<MemoryDB> {
        // if file is empty create a new db else load the db from the file
        let db = if self.data_file.metadata()?.len() == 0 {
            MemoryDB {
                memories: vec![],
                embeddings: Array2::zeros((0, Self::EMBEDDING_SIZE)),
            }
        } else {
            serde_json::from_reader(&self.data_file)?
        };
        Ok(db)
    }

    /// Save the `MemoryDB` to the given `File`.
    fn save_db(&mut self, db: &MemoryDB) -> Result<()> {
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
    const OPENAI_API_KEY_FILE_NAME: &str = "openai_api_key.txt";

    /// Load the `MemoryStore` from the default data file.
    ///
    /// The default data directory is set by the `MEM_DATA_DIR` environment variable.
    /// If this variable is not set, the default data directory is `~/.mem`.
    pub fn load() -> Result<MemoryStore> {
        let data_file = Self::default_data_file().context("Failed to load default data file.")?;
        let openai =
            Self::default_openai_client().context("Failed to load default OpenAI client.")?;
        Ok(Self::with_options(data_file, openai))
    }

    /// Create a new `MemoryStore` from the given `File`.
    ///
    /// This is useful for testing.
    pub fn with_options(data_file: File, openai: openai::Client) -> MemoryStore {
        MemoryStore { data_file, openai }
    }

    /// Get a handle to the default data file.
    pub fn default_data_file() -> Result<File> {
        let data_dir_path = Self::resolve_data_dir_path();
        let data_file_path = data_dir_path.join(Self::DATA_FILE_NAME);
        std::fs::create_dir_all(&data_dir_path).context(format!(
            "Failed to create data directory. Make sure you have write permissions to {}",
            data_dir_path.display()
        ))?;
        let data_file = OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .open(&data_file_path)
            .context(format!(
                "Failed to open data file. Make sure you have write permissions to {}",
                data_file_path.display()
            ))?;
        Ok(data_file)
    }

    /// Get the default OpenAI client.
    ///
    /// Uses the OpenAI API key stored in the `openai_api_key.txt` file in the data directory.
    pub fn default_openai_client() -> Result<openai::Client> {
        let openai_api_key_file_path =
            Self::resolve_data_dir_path().join(Self::OPENAI_API_KEY_FILE_NAME);
        let openai_api_key = std::fs::read_to_string(openai_api_key_file_path)
            .context("Failed to read OpenAI API key file. Did you set the OpenAI API key?")?;
        Ok(openai::Client::new(openai_api_key))
    }

    /// Store the OpenAI API key in the `openai_api_key.txt` file in the data directory.
    pub fn store_openai_api_key(openai_api_key: &str) -> Result<()> {
        let data_dir_path = Self::resolve_data_dir_path();
        let openai_api_key_file_path = data_dir_path.join(Self::OPENAI_API_KEY_FILE_NAME);
        std::fs::create_dir_all(&data_dir_path).context(format!(
            "Failed to create data directory. Make sure you have write permissions to {}",
            data_dir_path.display()
        ))?;
        std::fs::write(&openai_api_key_file_path, openai_api_key).context(format!(
            "Failed to write OpenAI API key file. Make sure you have write permissions to {}",
            openai_api_key_file_path.display()
        ))?;
        Ok(())
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
