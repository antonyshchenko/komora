use std::result;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum Error {
    #[error("Failed to serialize catalog metadata")]
    CatalogMetadataSerializationFailed(#[from] toml::ser::Error),

    #[error("Failed to deserialize catalog metadata")]
    CatalogMetadataDeserializationFailed(#[from] toml::de::Error),

    #[error("Failed to create catalog directory at {path}: {source}")]
    CatalogDirCreationFailed {
        path: String,
        #[source]
        source: std::io::Error,
    },

    #[error("Catalog already exists at {path}")]
    CatalogExists { path: String },

    #[error("Failed to write catalog metadata: {source}")]
    CatalogMetadataWriteFailed {
        #[source]
        source: std::io::Error,
    },

    #[error("Failed to read catalog metadata: {source}")]
    CatalogMetadataReadFailed {
        #[source]
        source: std::io::Error,
    },

    #[error("Incompatible catalog metadata version")]
    IncompatibleCatalogMetadataVersion,
}

pub type Result<T> = result::Result<T, Error>;
