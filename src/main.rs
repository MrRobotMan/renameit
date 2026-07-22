use std::{
    env,
    io::{self, IsTerminal},
    path::PathBuf,
};
use thiserror::Error;

use renameit_gui::{GuiError, run};
use renameit_lib::{
    DirectoryError, RenamerError,
    helpers::{get_directory, get_home},
};

fn main() -> Result<(), AnyError> {
    let path = if io::stdout().is_terminal() {
        match get_initial_directory() {
            Err(e) => return Err(AnyError::Lib(e.into())),
            Ok(p) => Some(p),
        }
    } else {
        None
    };
    Ok(run(path)?)
}

// Get the full path of a directory falling back to the home directory
// if nothing is provided. If the provided path is a file, the file's parent
// is returned.
fn get_initial_directory() -> Result<PathBuf, DirectoryError> {
    if let Some(arg) = env::args().nth(1) {
        get_directory(arg)
    } else {
        env::current_dir().or_else(|_| get_home())
    }
}

#[derive(Debug, Error)]
enum AnyError {
    #[error(transparent)]
    Gui(#[from] GuiError),
    #[error(transparent)]
    Lib(#[from] RenamerError),
}
