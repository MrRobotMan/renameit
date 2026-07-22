use std::io;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum FileError {
    #[error("File does not exist.")]
    NotFound,
    #[error("File does not have a stem.")]
    BadStem,
    #[error(transparent)]
    Io(#[from] io::Error),
}

#[derive(Debug, Error)]
pub enum DirectoryError {
    #[error(transparent)]
    Io(#[from] io::Error),
    #[error("No home directory could be found")]
    NoHome,
    #[error(transparent)]
    File(#[from] FileError),
    #[error("Path {0} does not exist.")]
    PathDoesNotExist(String),
    #[error("{0} is a file, not a directory.")]
    IsAFile(String),
}

#[derive(Debug, Error)]
pub enum RenamerError {
    #[error(transparent)]
    Directory(#[from] DirectoryError),
    #[error(transparent)]
    Io(#[from] io::Error),
    #[error(transparent)]
    File(#[from] FileError),
}
