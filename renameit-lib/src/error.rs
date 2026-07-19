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
