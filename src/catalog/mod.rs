use crate::error::{Error, Result};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, HashSet};

#[derive(Serialize, Deserialize, PartialEq, Debug)]
pub enum DbEngine {
    Komora,
}

#[derive(Serialize, Deserialize, PartialEq, Debug)]
pub enum StorageType {
    RowOriented,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Catalog {
    pub engine: DbEngine,
    pub storage_type: StorageType,
    pub version: u16,
    pub next_table_id: TableId,
    pub tables: BTreeMap<TableName, TableId>,
}

pub const CATALOG_LATEST_VERSION: u16 = 1;

impl Catalog {
    #[must_use]
    pub fn new() -> Self {
        Self {
            engine: DbEngine::Komora,
            storage_type: StorageType::RowOriented,
            version: CATALOG_LATEST_VERSION,
            next_table_id: TableId(1),
            tables: BTreeMap::new(),
        }
    }

    pub fn add_table(&mut self, name: TableName) -> Result<TableId> {
        // let name = name.clone();
        if self.tables.contains_key(&name) {
            return Err(Error::TableAlreadyExists { name });
        }

        let table_id = self.next_table_id;
        self.tables.insert(name, table_id);
        self.next_table_id = table_id.next();
        Ok(table_id)
    }

    pub fn remove_table(&mut self, name: &TableName) -> Result<TableId> {
        self.tables.remove(name).ok_or(Error::TableNotFound)
    }

    pub fn get_table_id(&self, name: &TableName) -> Result<TableId> {
        self.tables.get(name).copied().ok_or(Error::TableNotFound)
    }
}

impl Default for Catalog {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Serialize, Deserialize, PartialEq, Debug)]
pub enum ColumnType {
    Int,
    Text,
}

#[derive(Serialize, Deserialize, PartialEq, Debug)]
pub struct ColumnSchema {
    pub name: String,
    pub typ: ColumnType,
    pub nullable: bool,
}

impl ColumnSchema {
    #[must_use]
    pub fn new(name: &str, typ: ColumnType, nullable: bool) -> Self {
        Self { name: String::from(name), typ, nullable }
    }
}

#[derive(Serialize, Deserialize, PartialEq, Debug, Clone, Copy)]
pub struct TableId(pub u16);

impl TableId {
    #[must_use]
    pub fn next(self) -> Self {
        TableId(self.0 + 1)
    }
}

#[derive(Serialize, Deserialize, PartialEq, Debug, Ord, PartialOrd, Eq, Clone)]
pub struct TableName(pub String);

impl TableName {
    #[must_use]
    pub fn from(s: &str) -> TableName {
        TableName(String::from(s))
    }
}

#[derive(Serialize, Deserialize, PartialEq, Debug)]
pub struct TableSchema {
    pub columns: Vec<ColumnSchema>,
}

#[derive(Debug, Eq, PartialEq)]
pub enum TableSchemaError {
    InvalidTableName,
    InvalidColumnName { name: String },
    DuplicateColumn { name: String },
    MissingColumns,
}

impl TableSchema {
    #[must_use]
    pub fn validate(&self) -> Vec<TableSchemaError> {
        let mut errors = Vec::new();

        if self.columns.is_empty() {
            errors.push(TableSchemaError::MissingColumns);
        }

        let mut seen_column_names: HashSet<&String> = HashSet::new();

        for column in &self.columns {
            if seen_column_names.contains(&column.name) {
                errors.push(TableSchemaError::DuplicateColumn { name: column.name.clone() });
            }

            if column.name.is_empty() || !column.name.is_ascii() {
                errors.push(TableSchemaError::InvalidColumnName { name: column.name.clone() });
            }

            seen_column_names.insert(&column.name);
        }

        errors
    }
}

#[derive(Serialize, Deserialize, PartialEq, Debug)]
pub struct Table {
    pub id: TableId,
    pub name: TableName,
    pub schema: TableSchema,
}
