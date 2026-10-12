use std::{
    env,
    io::{self, IsTerminal},
    path::PathBuf,
};

use eros::{ReshapeUnion, Result};

use renameit_gui::run;
use renameit_lib::{
    IsAFileError, NoHomeDirError, PathDoesNotExistError,
    helpers::{get_directory, get_start_dir},
};

fn main() -> Result<()> {
    let path = if io::stdout().is_terminal() {
        Some(get_initial_directory()?)
    } else {
        None
    };
    run(path)
}

// Get the full path of a directory falling back to the home directory
// if nothing is provided. If the provided path is a file, the file's parent
// is returned.
fn get_initial_directory() -> Result<
    PathBuf,
    (
        NoHomeDirError,
        PathDoesNotExistError,
        IsAFileError,
        std::io::Error,
    ),
> {
    env::args().nth(1).map_or_else(
        || env::current_dir().or_else(|_| get_start_dir().widen()),
        |arg| get_directory(arg).widen(),
    )
}
