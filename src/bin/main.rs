use clap::{Parser, Subcommand};
use env_logger::Env;
use komora::catalog;
use komora::error::Result;
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
    Doctor,
}

fn main() {
    env_logger::init_from_env(Env::default().filter("LOG_LEVEL"));

    let cli = Cli::parse();
    let catalog_path = Path::new(&cli.db);

    if let Err(error) = match &cli.command {
        Commands::Init => init(&catalog_path),
        Commands::Doctor => doctor(&catalog_path),
    } {
        eprintln!("{}", error);
        exit(1);
    }
}

fn init(catalog_path: &Path) -> Result<()> {
    log::debug!("Initializing in {:?}", catalog_path);
    catalog::create_in_dir(catalog_path)?;
    log::debug!("Done");
    Ok(())
}

fn doctor(catalog_path: &Path) -> Result<()> {
    log::debug!("Checking catalog in {:?}", catalog_path);
    let metadata = catalog::read_catalog_metadata(catalog_path)?;
    println!("Catalog is valid, version: {}", metadata.version);
    Ok(())
}
