use std::{
    env,
    ffi::OsStr,
    fs::canonicalize,
    path::{Path, PathBuf},
};

use home;

use crate::DirectoryError;

#[derive(Debug)]
pub(crate) enum PathString {
    Valid(String),
    Invalid(String),
}

/// Convert a Path to a mutable string
pub(crate) fn generate_path_as_string(part: Option<&OsStr>) -> Option<PathString> {
    part.map(|path| match path.to_str() {
        Some(s) => PathString::Valid(s.into()),
        None => PathString::Invalid(path.to_string_lossy().into_owned()),
    })
}

pub fn get_home() -> Result<PathBuf, DirectoryError> {
    match env::current_dir() {
        Ok(dir) => Ok(dir),
        Err(_) => {
            let d = home::home_dir();
            match d {
                Some(path) => Ok(path),
                None => Err(DirectoryError::NoHome),
            }
        }
    }
}

/// Get the full path of a directory.
/// If the provided path is a file, the file's parent is returned.
pub fn get_directory<P: AsRef<Path>>(path: P) -> Result<PathBuf, DirectoryError> {
    if !PathBuf::from(path.as_ref()).exists() {
        return Err(DirectoryError::PathDoesNotExist(
            path.as_ref().display().to_string(),
        ));
    }
    let p = canonicalize(path)?;
    if p.is_file() {
        Err(DirectoryError::IsAFile(p.display().to_string()))
    } else {
        Ok(p)
    }
}
