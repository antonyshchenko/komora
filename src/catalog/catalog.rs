use crate::catalog::metadata::{CATALOG_METADATA_FILE_NAME, CatalogMetadata};
use crate::error::{Error, Result};
use std::fs;
use std::path::Path;

pub fn create_in_dir(dir: &Path) -> Result<()> {
    let metadata_path = &dir.join(CATALOG_METADATA_FILE_NAME);

    if metadata_path.exists() {
        return Err(Error::CatalogExists {
            path: dir.to_string_lossy().to_string(),
        });
    }

    fs::create_dir_all(dir).map_err(|err| Error::CatalogDirCreationFailed {
        path: dir.to_string_lossy().to_string(),
        source: err,
    })?;

    let metadata = CatalogMetadata::new();
    metadata.write(metadata_path)?;

    Ok(())
}

pub fn read_catalog_metadata(dir: &Path) -> Result<CatalogMetadata> {
    CatalogMetadata::read_from_dir(dir)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::catalog::metadata::{CATALOG_METADATA_LATEST_VERSION, DbEngine};
    use std::path::PathBuf;
    use tempfile::tempdir;

    #[test]
    fn catalog_dir_creation() {
        let dir = tempdir().unwrap();

        create_in_dir(dir.path()).unwrap();
    }

    #[test]
    fn not_allowed_to_create_catalog_dir_twice() {
        let dir = tempdir().unwrap();

        create_in_dir(dir.path()).unwrap();

        assert!(matches!(
            create_in_dir(dir.path()).unwrap_err(),
            Error::CatalogExists { .. }
        ));
    }

    #[test]
    fn catalog_metadata_reading_success() {
        let dir = tempdir().unwrap();

        create_in_dir(dir.path()).unwrap();

        let metadata = read_catalog_metadata(dir.path()).unwrap();
        assert_eq!(metadata.version, CATALOG_METADATA_LATEST_VERSION);
        assert_eq!(metadata.engine, DbEngine::Komora);
    }

    #[test]
    fn catalog_metadata_reading_failure() {
        let dir = tempdir().unwrap();

        create_in_dir(dir.path()).unwrap();
        fs::remove_file(dir.path().join(CATALOG_METADATA_FILE_NAME)).unwrap();

        assert!(matches!(
            read_catalog_metadata(dir.path()).unwrap_err(),
            Error::CatalogMetadataReadFailed { .. }
        ));
    }

    #[test]
    fn catalog_metadata_reading_failure_due_to_incompatible_version() {
        let dir = catalog_fixture_path("incompatible_version");

        assert!(matches!(
            read_catalog_metadata(&dir).unwrap_err(),
            Error::IncompatibleCatalogMetadataVersion { .. }
        ));
    }

    #[test]
    fn catalog_metadata_deserialization_failure() {
        let dir = catalog_fixture_path("invalid_metadata");

        assert!(matches!(
            read_catalog_metadata(&dir).unwrap_err(),
            Error::CatalogMetadataDeserializationFailed { .. }
        ));
    }

    fn catalog_fixture_path(name: &str) -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("tests")
            .join("fixtures")
            .join("catalog")
            .join(name)
    }
}
