use crate::catalog::{TableName, TableSchemaError};
use std::result;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum Error {
    #[error("Failed to serialize catalog")]
    CatalogSerializationFailed(#[from] toml::ser::Error),

    #[error("Failed to deserialize catalog")]
    CatalogDeserializationFailed(#[from] toml::de::Error),

    #[error("Failed to create catalog directory at {path}: {source}")]
    CatalogDirCreationFailed {
        path: String,
        #[source]
        source: std::io::Error,
    },

    #[error("Catalog already exists at {path}")]
    CatalogAlreadyExists { path: String },

    #[error("Failed to write catalog: {source}")]
    CatalogWriteFailed {
        #[source]
        source: std::io::Error,
    },

    #[error("Failed to read catalog: {source}")]
    CatalogReadFailed {
        #[source]
        source: std::io::Error,
    },

    #[error("Incompatible catalog version")]
    IncompatibleCatalogVersion,

    #[error("Invalid table name")]
    InvalidTableName { name: TableName },

    #[error("Invalid table schema")]
    InvalidTableSchema { name: TableName, errors: Vec<TableSchemaError> },

    #[error("Table already exists")]
    TableAlreadyExists { name: TableName },

    #[error("Table not found in catalog")]
    TableNotFound,

    #[error("Table data not found")]
    TableDataNotFound,

    #[error("Failed to write table data: {source}")]
    TableDataWriteFailed {
        #[source]
        source: std::io::Error,
    },

    #[error("Failed to read table data: {source}")]
    TableDataReadFailed {
        #[source]
        source: std::io::Error,
    },
}

pub type Result<T> = result::Result<T, Error>;
