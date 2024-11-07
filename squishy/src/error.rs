use std::path::PathBuf;

use thiserror::Error;

#[derive(Error, Debug)]
pub enum SquishyError {
    #[error("Failed to find SquashFS magic bytes in the file")]
    NoSquashFsFound,

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("SquashFS error: {0}")]
    InvalidSquashFS(String),

    #[error("Symlink error: {0}")]
    SymlinkError(String),

    #[error("File not found: {0}")]
    FileNotFound(PathBuf),
}
