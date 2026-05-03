use crate::catalog::{
    CATALOG_LATEST_VERSION, Catalog, StorageType, Table, TableId, TableName, TableSchema,
};
use crate::error::{Error, Result};
use crate::utils;
use std::fs;
use std::fs::File;
use std::path::PathBuf;

#[derive(Debug)]
pub struct Storage {
    pub dir: PathBuf,
}

#[derive(Debug)]
pub struct StorageMetaInfo {
    pub storage_type: StorageType,
    pub catalog_version: u16,
}

#[derive(Debug, Clone)]
pub struct Rid {
    pub page_id: u32,
    pub slot_id: u32,
}

#[derive(Debug)]
pub enum TupleElement {
    Null,
    Int(i32),
    Text(String),
}

pub type Tuple = Vec<TupleElement>;

pub const CATALOG_FILE_NAME: &str = "catalog.toml";

impl Storage {
    pub fn new(dir: impl Into<PathBuf>) -> Self {
        Storage { dir: dir.into() }
    }

    pub fn create_catalog(&self) -> Result<()> {
        log::debug!("Creating catalog in {}", self.dir.display());

        let catalog_path = self.dir.join(CATALOG_FILE_NAME);
        if catalog_path.exists() {
            return Err(Error::CatalogAlreadyExists {
                path: self.dir.to_string_lossy().to_string(),
            });
        }

        fs::create_dir_all(&self.dir)
            .and_then(|()| fs::create_dir_all(self.tables_dir()))
            .map_err(|err| Error::CatalogDirCreationFailed {
                path: self.dir.to_string_lossy().to_string(),
                source: err,
            })?;

        let catalog = Catalog::new();
        self.write_catalog(&catalog)?;

        log::debug!("Catalog created");

        Ok(())
    }

    fn write_catalog(&self, catalog: &Catalog) -> Result<()> {
        let catalog_path = self.dir.join(CATALOG_FILE_NAME);
        log::debug!("Writing catalog to {}", catalog_path.display());

        let serialized = toml::to_string(catalog)?;
        utils::fs::atomic_write(catalog_path, serialized.as_bytes())
            .map_err(|e| Error::CatalogWriteFailed { source: e })?;

        Ok(())
    }

    fn read_catalog(&self) -> Result<Catalog> {
        let catalog_path = self.dir.join(CATALOG_FILE_NAME);
        log::debug!("Reading catalog from {}", catalog_path.display());

        let serialized =
            fs::read_to_string(catalog_path).map_err(|e| Error::CatalogReadFailed { source: e })?;

        let catalog: Catalog = toml::from_str(&serialized)?;
        if catalog.version > CATALOG_LATEST_VERSION {
            return Err(Error::IncompatibleCatalogVersion);
        }

        Ok(catalog)
    }

    pub fn get_meta_info(&self) -> Result<StorageMetaInfo> {
        let catalog = self.read_catalog()?;

        Ok(StorageMetaInfo { storage_type: catalog.storage_type, catalog_version: catalog.version })
    }

    pub fn create_table(&self, name: &TableName, schema: &TableSchema) -> Result<()> {
        if name.0.is_empty() || !name.0.is_ascii() {
            return Err(Error::InvalidTableName { name: name.clone() });
        }

        let schema_errors = schema.validate();
        if !schema_errors.is_empty() {
            return Err(Error::InvalidTableSchema { name: name.clone(), errors: schema_errors });
        }

        let mut catalog = self.read_catalog()?;
        let table_id = catalog.add_table(name.clone())?;

        self.write_table_schema(table_id, schema)?;

        // We are just creating an empty data file for now
        File::create(self.table_data_path(table_id))
            .and_then(|file| file.sync_all())
            .map_err(|e| Error::TableDataWriteFailed { source: e })?;

        self.write_catalog(&catalog)?;

        Ok(())
    }

    pub fn drop_table(&self, name: &TableName) -> Result<()> {
        let mut catalog = self.read_catalog()?;
        let table_id = catalog.remove_table(name)?;

        self.write_catalog(&catalog)?;
        self.remove_table_schema(table_id)?;
        self.remove_table_data(table_id)?;

        Ok(())
    }

    pub fn list_tables(&self) -> Result<Vec<TableName>> {
        let catalog = self.read_catalog()?;
        Ok(catalog.tables.keys().cloned().collect())
    }

    pub fn describe_table(&self, name: &TableName) -> Result<TableSchema> {
        Ok(self.open_table(name)?.schema)
    }

    pub fn open_table(&self, name: &TableName) -> Result<Table> {
        let catalog = self.read_catalog()?;
        let table_id = catalog.get_table_id(name)?;
        let table_schema = self.read_table_schema(table_id)?;

        if !self.table_data_path(table_id).exists() {
            return Err(Error::TableDataNotFound);
        }

        Ok(Table { id: table_id, name: name.clone(), schema: table_schema })
    }

    #[allow(clippy::unused_self)]
    pub fn get(&self, _table: &Table, _rid: &Rid) -> Result<Tuple> {
        todo!("later")
    }

    pub fn insert(&self, _table: &Table, _tuple: &Tuple) -> Result<Rid> {
        todo!("later")
    }

    pub fn update(&self, _table: &Table, _rid: &Rid, _tuple: &Tuple) -> Result<()> {
        todo!("later")
    }

    pub fn delete(&self, _table: &Table, _rid: &Rid) -> Result<()> {
        todo!("later")
    }

    #[allow(clippy::unused_self)]
    #[allow(clippy::unnecessary_wraps)]
    pub fn scan(&self, _table: &Table) -> Result<impl Iterator<Item = &Tuple> + '_> {
        Ok(std::iter::empty())
    }

    fn read_table_schema(&self, table_id: TableId) -> Result<TableSchema> {
        let serialized = fs::read_to_string(self.table_schema_path(table_id))
            .map_err(|e| Error::CatalogReadFailed { source: e })?;

        Ok(toml::from_str(&serialized)?)
    }

    fn write_table_schema(&self, table_id: TableId, table_schema: &TableSchema) -> Result<()> {
        let serialized = toml::to_string(&table_schema)?;
        utils::fs::atomic_write(self.table_schema_path(table_id), serialized.as_bytes())
            .map_err(|e| Error::CatalogWriteFailed { source: e })?;

        Ok(())
    }

    fn remove_table_schema(&self, table_id: TableId) -> Result<()> {
        fs::remove_file(self.table_schema_path(table_id))
            .map_err(|e| Error::CatalogWriteFailed { source: e })
    }

    fn remove_table_data(&self, table_id: TableId) -> Result<()> {
        fs::remove_file(self.table_data_path(table_id))
            .map_err(|e| Error::TableDataWriteFailed { source: e })
    }

    fn tables_dir(&self) -> PathBuf {
        self.dir.join("tables")
    }

    fn table_schema_path(&self, table_id: TableId) -> PathBuf {
        self.tables_dir().join(format!("{}.schema.toml", table_id.0))
    }

    fn table_data_path(&self, table_id: TableId) -> PathBuf {
        self.tables_dir().join(format!("{}.data", table_id.0))
    }
}

#[cfg(test)]
mod tests {
    use crate::catalog::{
        CATALOG_LATEST_VERSION, ColumnSchema, ColumnType, DbEngine, TableId, TableName,
        TableSchema, TableSchemaError,
    };
    use crate::error::Error;
    use crate::storage::{CATALOG_FILE_NAME, Storage};
    use assert_matches::assert_matches;
    use std::fs;
    use std::path::PathBuf;
    use tempfile::tempdir;

    #[test]
    fn catalog_creation() {
        let dir = tempdir().unwrap();

        Storage::new(dir.path()).create_catalog().unwrap();
    }

    #[test]
    fn not_allowed_to_create_catalog_twice() {
        let storage = Storage::new(tempdir().unwrap().path());

        storage.create_catalog().unwrap();

        assert_matches!(storage.create_catalog().unwrap_err(), Error::CatalogAlreadyExists { .. });
    }

    #[test]
    fn catalog_reading_success() {
        let storage = Storage::new(tempdir().unwrap().path());
        storage.create_catalog().unwrap();

        let metadata = storage.read_catalog().unwrap();

        assert_eq!(metadata.version, CATALOG_LATEST_VERSION);
        assert_eq!(metadata.engine, DbEngine::Komora);
    }

    #[test]
    fn catalog_reading_failure() {
        let dir = tempdir().unwrap();
        let storage = Storage::new(dir.path());
        storage.create_catalog().unwrap();

        fs::remove_file(dir.path().join(CATALOG_FILE_NAME)).unwrap();

        assert_matches!(storage.read_catalog().unwrap_err(), Error::CatalogReadFailed { .. });
    }

    #[test]
    fn catalog_reading_failure_due_to_incompatible_version() {
        let storage = Storage::new(catalog_fixture_path("incompatible_version"));

        assert_matches!(
            storage.read_catalog().unwrap_err(),
            Error::IncompatibleCatalogVersion { .. }
        );
    }

    #[test]
    fn catalog_deserialization_failure() {
        let storage = Storage::new(catalog_fixture_path("invalid_metadata"));

        assert_matches!(
            storage.read_catalog().unwrap_err(),
            Error::CatalogDeserializationFailed { .. }
        );
    }

    #[test]
    fn table_creation_success() {
        let storage = Storage::new(tempdir().unwrap().path());
        storage.create_catalog().unwrap();
        let table1_name = TableName::from("table1");
        let table1_schema = table1_schema();

        assert!(storage.list_tables().unwrap().is_empty());

        storage.create_table(&table1_name, &table1_schema).unwrap();

        assert!(storage.list_tables().unwrap().contains(&table1_name));

        let table1 = storage.open_table(&table1_name).unwrap();
        assert_eq!(table1_name, table1.name);
        assert!(storage.table_schema_path(table1.id).exists());
        assert!(storage.table_data_path(table1.id).exists());
    }

    #[test]
    fn table_creation_with_duplicate_name() {
        let storage = Storage::new(tempdir().unwrap().path());
        storage.create_catalog().unwrap();
        let table1_name = TableName::from("table1");
        let table1_schema = table1_schema();
        storage.create_table(&table1_name, &table1_schema).unwrap();

        assert_matches!(
            storage
                .create_table(&table1_name, &table1_schema)
                .unwrap_err(),
            Error::TableAlreadyExists { name } if name == table1_name
        );
    }

    #[test]
    fn table_creation_without_columns() {
        let storage = Storage::new(tempdir().unwrap().path());
        storage.create_catalog().unwrap();

        let table_name = TableName::from("table1");
        let table_schema = TableSchema { columns: vec![] };

        let (name, errors) = assert_matches!(
            storage.create_table(&table_name, &table_schema).unwrap_err(),
            Error::InvalidTableSchema { name, errors } => (name, errors)
        );
        assert_eq!(table_name, name);
        assert_eq!(vec![TableSchemaError::MissingColumns], errors);
    }

    #[test]
    fn table_creation_with_invalid_name() {
        let storage = Storage::new(tempdir().unwrap().path());
        storage.create_catalog().unwrap();
        let table_schema = table2_schema();

        let empty_name = TableName::from("");
        assert_matches!(
            storage.create_table(&empty_name, &table_schema).unwrap_err(),
            Error::InvalidTableName { name } if name == empty_name
        );

        let non_ascii_name = TableName::from("таблиця");
        assert_matches!(
            storage.create_table(&non_ascii_name, &table_schema).unwrap_err(),
            Error::InvalidTableName { name } if name == non_ascii_name
        );
    }

    #[test]
    fn table_creation_with_invalid_column_name() {
        let storage = Storage::new(tempdir().unwrap().path());
        storage.create_catalog().unwrap();
        let table_name = TableName::from("table1");

        let table_schema =
            TableSchema { columns: vec![ColumnSchema::new("", ColumnType::Int, false)] };

        let (name, errors) = assert_matches!(
            storage.create_table(&table_name, &table_schema).unwrap_err(),
            Error::InvalidTableSchema { name, errors } => (name, errors)
        );

        assert_eq!(TableName::from("table1"), name);
        assert_eq!(vec![TableSchemaError::InvalidColumnName { name: String::from("") }], errors);

        let table_schema = TableSchema {
            columns: vec![ColumnSchema::new("колонка", ColumnType::Int, false)],
        };

        let (name, errors) = assert_matches!(
            storage.create_table(&table_name, &table_schema).unwrap_err(),
            Error::InvalidTableSchema { name, errors } => (name, errors)
        );

        assert_eq!(TableName::from("table1"), name);
        assert_eq!(
            vec![TableSchemaError::InvalidColumnName { name: String::from("колонка") }],
            errors
        );
    }

    #[test]
    fn table_creation_with_duplicate_column_name() {
        let storage = Storage::new(tempdir().unwrap().path());
        storage.create_catalog().unwrap();
        let table_name = TableName::from("table1");

        let table_schema = TableSchema {
            columns: vec![
                ColumnSchema::new("column1", ColumnType::Int, false),
                ColumnSchema::new("column2", ColumnType::Text, false),
                ColumnSchema::new("column1", ColumnType::Text, false),
            ],
        };

        let (name, errors) = assert_matches!(
            storage.create_table(&table_name, &table_schema).unwrap_err(),
            Error::InvalidTableSchema { name, errors } => (name, errors)
        );

        assert_eq!(TableName::from("table1"), name);
        assert_eq!(
            vec![TableSchemaError::DuplicateColumn { name: String::from("column1") }],
            errors
        );
    }

    #[test]
    fn drop_table_success() {
        let storage = Storage::new(tempdir().unwrap().path());
        storage.create_catalog().unwrap();
        let table1_name = TableName::from("table1");
        let table1_schema = table1_schema();

        storage.create_table(&table1_name, &table1_schema).unwrap();
        assert!(storage.list_tables().unwrap().contains(&table1_name));
        let table_id = storage.open_table(&table1_name).unwrap().id;

        storage.drop_table(&table1_name).unwrap();

        assert!(storage.list_tables().unwrap().is_empty());
        assert!(!storage.table_schema_path(table_id).exists());
        assert!(!storage.table_data_path(table_id).exists());
    }

    #[test]
    fn drop_non_existing_table() {
        let storage = Storage::new(tempdir().unwrap().path());
        storage.create_catalog().unwrap();

        assert_matches!(
            storage.drop_table(&TableName::from("table1")).unwrap_err(),
            Error::TableNotFound
        );
    }

    #[test]
    fn open_table_success() {
        let storage = Storage::new(catalog_fixture_path("one_empty_table"));
        let expected_table_name = TableName::from("table2");
        let expected_schema = table2_schema();

        let table = storage.open_table(&expected_table_name).unwrap();
        assert_eq!(TableId(1), table.id);
        assert_eq!(expected_table_name, table.name);
        assert_eq!(expected_schema, table.schema);
    }

    #[test]
    fn open_non_existent_table() {
        let storage = Storage::new(catalog_fixture_path("one_empty_table"));

        assert_matches!(
            storage.open_table(&TableName::from("table123")).unwrap_err(),
            Error::TableNotFound
        );
    }

    #[test]
    fn open_table_with_corrupted_metadata() {
        let storage = Storage::new(catalog_fixture_path("corrupted_table_metadata"));

        assert_matches!(
            storage.open_table(&TableName::from("table1")).unwrap_err(),
            Error::CatalogDeserializationFailed { .. }
        );
    }

    #[test]
    fn describe_table_success() {
        let storage = Storage::new(catalog_fixture_path("one_empty_table"));
        let expected_table_name = TableName::from("table2");
        let expected_schema = table2_schema();

        assert_eq!(expected_schema, storage.describe_table(&expected_table_name).unwrap());
    }

    #[test]
    fn describe_non_existent_table() {
        let storage = Storage::new(catalog_fixture_path("one_empty_table"));

        assert_matches!(
            storage.describe_table(&TableName::from("table123")).unwrap_err(),
            Error::TableNotFound
        );
    }

    #[test]
    fn describe_table_with_corrupted_metadata() {
        let storage = Storage::new(catalog_fixture_path("corrupted_table_metadata"));

        assert_matches!(
            storage.describe_table(&TableName::from("table1")).unwrap_err(),
            Error::CatalogDeserializationFailed { .. }
        );
    }

    #[test]
    fn list_tables() {
        let storage = Storage::new(tempdir().unwrap().path());
        storage.create_catalog().unwrap();

        assert!(storage.list_tables().unwrap().is_empty());

        let table1_name = TableName::from("table1");
        let table1_schema = table1_schema();

        let table2_name = TableName::from("table2");
        let table2_schema = table2_schema();

        storage.create_table(&table1_name, &table1_schema).unwrap();
        storage.create_table(&table2_name, &table2_schema).unwrap();

        assert_eq!(vec![table1_name, table2_name], storage.list_tables().unwrap());
    }

    fn table1_schema() -> TableSchema {
        TableSchema { columns: vec![ColumnSchema::new("column1", ColumnType::Int, false)] }
    }

    fn table2_schema() -> TableSchema {
        TableSchema {
            columns: vec![
                ColumnSchema::new("column1", ColumnType::Int, false),
                ColumnSchema::new("column2", ColumnType::Text, true),
            ],
        }
    }

    fn catalog_fixture_path(name: &str) -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("tests")
            .join("fixtures")
            .join("catalog")
            .join(name)
    }
}
