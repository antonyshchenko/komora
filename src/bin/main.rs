use clap::{Parser, Subcommand};
use env_logger::Env;
use komora::error::Result;
use komora::storage::Storage;
use std::path::Path;
use std::process::exit;

#[derive(Parser)]
#[command(version, about, long_about = None)]
#[command(propagate_version = true)]
struct Cli {
    #[arg(short, long)]
    db: String,

    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    Init,
    Info,
}

fn main() {
    env_logger::init_from_env(Env::default().filter("LOG_LEVEL"));

    match Cli::try_parse() {
        Ok(cli) => {
            let dir = Path::new(&cli.db);

            if let Err(error) = match &cli.command {
                Commands::Init => init(dir),
                Commands::Info => info(dir),
            } {
                eprintln!("{}", error);
                exit(1);
            }
        }
        Err(error) => error.exit(),
    }
}

fn init(dir: &Path) -> Result<()> {
    Storage::new(dir).create_catalog()?;
    Ok(())
}

fn info(dir: &Path) -> Result<()> {
    let meta_info = Storage::new(dir).get_meta_info()?;
    println!("Storage type: {:?}", meta_info.storage_type);
    println!("Catalog version: {}", meta_info.catalog_version);
    Ok(())
}
