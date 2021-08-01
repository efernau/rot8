//! Error types for rot8
//!
//! Since we handle numerous types of error cases,
//! this will probably be expanded as-needed.

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("Invalid degree of rotation, +/- 90 degrees only, got {0}")]
    InvalidDegrees(isize),

    #[error("Underlying I/O error")]
    IOError(#[from] std::io::Error),

    #[error("Unknown Error. Sorry!")]
    Unknown,
}

pub type Result<T> = std::result::Result<T, Error>;
