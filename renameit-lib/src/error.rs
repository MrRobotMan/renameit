use thiserror::Error;

#[derive(Debug, Error)]
#[error("File does not exist.")]
pub struct FileNotFoundError;

#[derive(Debug, Error)]
#[error("File does not have a stem.")]
pub struct BadFileStemError;

#[derive(Debug, Error)]
#[error("No home directory could be found")]
pub struct NoHomeDirError;

#[derive(Debug, Error)]
#[error("Path {0} does not exist.")]
pub struct PathDoesNotExistError(pub String);

#[derive(Debug, Error)]
#[error("{0} is a file, not a directory.")]
pub struct IsAFileError(pub String);
