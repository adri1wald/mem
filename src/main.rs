use clap::{Parser, Subcommand};
use std::io::{stdin, stdout, Write};

mod store;

use store::MemoryStore;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct MemCli {
    #[command(subcommand)]
    command: MemCommand,
}

#[derive(Subcommand, Debug)]
enum MemCommand {
    /// Insert a memory into the store
    Insert {
        /// The memory to store
        #[arg(value_name = "MEMORY")]
        mem: String,
        /// A description of the memory that is used for semantic retrieval
        #[arg(value_name = "DESCRIPTION")]
        description: String,
    },
    /// Get a memory from the store
    Get {
        /// A description of the memory you are looking for
        #[arg(value_name = "DESCRIPTION")]
        description: String,
    },
    /// List memories from the store
    List {
        /// The maximum number of memories to list
        #[arg(short, long, value_name = "COUNT", default_value_t = 10)]
        count: u8,
        /// A description of the memory you are looking for
        #[arg(value_name = "DESCRIPTION")]
        description: String,
    },
    /// Set OpenAI API key
    SetKey,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli = MemCli::parse();

    match &cli.command {
        MemCommand::Insert { mem, description } => {
            let mut store = MemoryStore::load()?;
            store.insert(mem, description)?;
            println!("Memory inserted!");
        }
        MemCommand::Get { description } => {
            let store = MemoryStore::load()?;
            let memory = store.get(description)?;
            if let Some(memory) = memory {
                println!(
                    "[{score}] {memory}",
                    memory = memory.value,
                    score = format!("{:.2}", memory.score)
                );
            } else {
                println!("No memory found!");
            }
        }
        MemCommand::List { description, count } => {
            let store = MemoryStore::load()?;
            let memories = store.list(description, *count as usize)?;
            if memories.is_empty() {
                println!("No memories found!");
            } else {
                memories.iter().for_each(|memory| {
                    println!(
                        "[{score}] {memory}",
                        memory = memory.value,
                        score = format!("{:.2}", memory.score)
                    );
                });
            }
        }
        MemCommand::SetKey => {
            print!("Please enter your API key: ");
            let _ = stdout().flush();
            let mut key = String::new();
            stdin().read_line(&mut key)?;
            key = key.trim_end().to_string();
            MemoryStore::store_openai_api_key(&key)?;
            println!("Key set!");
        }
    }
    Ok(())
}
