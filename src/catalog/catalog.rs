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
    use crate::error::Result;
    use tempfile::tempdir;

    #[test]
    fn catalog_dir_creation() -> Result<()> {
        let dir = tempdir().unwrap();

        create_in_dir(dir.path())?;

        Ok(())
    }

    #[test]
    fn catalog_dir_cant_be_created_twice() -> Result<()> {
        let dir = tempdir().unwrap();

        create_in_dir(dir.path())?;
        assert!(create_in_dir(dir.path()).is_err());

        Ok(())
    }

    #[test]
    fn catalog_metadata_reading_success() -> Result<()> {
        let dir = tempdir().unwrap();

        create_in_dir(dir.path())?;
        let metadata = read_catalog_metadata(dir.path())?;
        assert_eq!(metadata.version, CATALOG_METADATA_LATEST_VERSION);
        assert_eq!(metadata.engine, DbEngine::Komora);

        Ok(())
    }

    #[test]
    fn catalog_metadata_reading_failure() -> Result<()> {
        let dir = tempdir().unwrap();

        create_in_dir(dir.path())?;
        fs::remove_file(dir.path().join(CATALOG_METADATA_FILE_NAME)).unwrap();
        assert!(read_catalog_metadata(dir.path()).is_err());

        Ok(())
    }
}
