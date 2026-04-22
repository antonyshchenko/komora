use crate::error::{Error, Result};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;

#[derive(Serialize, Deserialize, PartialEq, Debug)]
pub enum DbEngine {
    Komora,
}

pub const CATALOG_METADATA_LATEST_VERSION: u16 = 1;
pub const CATALOG_METADATA_FILE_NAME: &str = "catalog_metadata.toml";

#[derive(Serialize, Deserialize, Debug)]
pub struct CatalogMetadata {
    pub engine: DbEngine,
    pub version: u16,
}

impl CatalogMetadata {
    pub fn new() -> Self {
        Self {
            engine: DbEngine::Komora,
            version: CATALOG_METADATA_LATEST_VERSION,
        }
    }

    pub fn write(self, file_name: &Path) -> Result<()> {
        let serialized = toml::to_string(&self)?;
        fs::write(file_name, serialized)
            .map_err(|e| Error::CatalogMetadataWriteFailed { source: e })?;
        Ok(())
    }

    pub fn read_from_dir(dir: &Path) -> Result<Self> {
        let serialized = fs::read_to_string(dir.join(CATALOG_METADATA_FILE_NAME))
            .map_err(|e| Error::CatalogMetadataReadFailed { source: e })?;

        let metadata: CatalogMetadata = toml::from_str(&serialized)?;
        if metadata.version > CATALOG_METADATA_LATEST_VERSION {
            return Err(Error::IncompatibleCatalogMetadataVersion);
        }

        Ok(metadata)
    }
}
