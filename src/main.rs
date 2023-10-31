use clap::{Parser, Subcommand};
use std::env;
use std::path::PathBuf;

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
}

const MEM_DATA_DIR_ENV_VAR: &str = "MEM_DATA_DIR";

fn resolve_data_dir() -> PathBuf {
    if let Ok(data_dir) = env::var(MEM_DATA_DIR_ENV_VAR) {
        PathBuf::from(data_dir)
    } else {
        let home_dir = dirs::home_dir().expect(&format!(
            "Unable to determine home directory. Set the {} environment variable to override.",
            MEM_DATA_DIR_ENV_VAR
        ));
        home_dir.join(".mem")
    }
}

fn main() {
    let cli = MemCli::parse();
    let data_dir = resolve_data_dir();
    match &cli.command {
        MemCommand::Insert { mem, description } => {
            println!("Inserting memory: {}", mem);
            println!("Description: {}", description);
            println!("Data dir: {:?}", data_dir);
        }
        MemCommand::Get { description } => {
            println!("Getting memory: {}", description);
            println!("Data dir: {:?}", data_dir);
        }
    }
}
