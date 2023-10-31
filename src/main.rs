use clap::{Parser, Subcommand};

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
        #[arg(short, long, value_name = "MEMORY")]
        mem: String,
        /// A description of the memory that is used for semantic retrieval
        #[arg(short, long, value_name = "DESCRIPTION")]
        description: String,
    },
    /// Get a memory from the store
    Get {
        /// A description of the memory you are looking for
        #[arg(short, long, value_name = "DESCRIPTION")]
        description: String,
    },
    /// List memories from the store
    List {
        /// A description of the memory you are looking for
        #[arg(short, long, value_name = "DESCRIPTION")]
        description: String,
        /// The maximum number of memories to list
        #[arg(short, long, value_name = "COUNT", default_value_t = 10)]
        count: u8,
    },
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli = MemCli::parse();
    let mut store = MemoryStore::load()?;

    match &cli.command {
        MemCommand::Insert { mem, description } => {
            store.insert(mem, description)?;
            println!("Memory inserted!");
        }
        MemCommand::Get { description } => {
            let memory = store.get(description)?;
            if let Some(memory) = memory {
                println!("{memory}");
            } else {
                println!("No memory found!");
            }
        }
        MemCommand::List { description, count } => {
            let memories = store.list(description, *count as usize)?;
            if memories.is_empty() {
                println!("No memories found!");
            } else {
                memories.iter().for_each(|memory| println!("{memory}"));
            }
        }
    }
    Ok(())
}
