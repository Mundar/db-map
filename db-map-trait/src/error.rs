//! # Error handling for the `DBMap` trait.
use std::io;

/// The standard error type for the `DBMap` trait.
#[derive(Debug, thiserror::Error)]
pub enum Error {
    /// Passthrough [`std::io::Error`]
    #[error(transparent)]
    IoError(#[from] io::Error),

    /// Passthrough database error.
    #[error(transparent)]
    DBError(#[from] anyhow::Error),
}

/// The standard result type for the `DBMap` trait.
pub type Result<T> = std::result::Result<T, Error>;
