use std::fs::File;
use std::io::Write;
use std::path::PathBuf;
use std::{fs, io};

pub fn atomic_write(file_name: impl Into<PathBuf>, data: &[u8]) -> io::Result<()> {
    let file_name = file_name.into();
    let tmp_file_name = file_name.with_extension(".tmp");
    let mut tmp_file = File::create(&tmp_file_name)?;
    tmp_file.write_all(data)?;
    tmp_file.sync_all()?;
    fs::rename(&tmp_file_name, file_name)?;

    Ok(())
}
