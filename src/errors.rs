use std::{io, string};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error("invalid UUID: {0}")]
    InvalidUuid(#[from] uuid::Error),

    #[error("io failure: {0}")]
    IO(#[from] io::Error),

    #[error("malformed UTF-16 string: {0}")]
    Utf16Error(#[from] string::FromUtf16Error),

    #[error("UTF-16 string is missing null terminator: {0}")]
    Utf16MissingNull(#[from] widestring::MissingNulError<u16>),
}

pub type Result<T> = std::result::Result<T, Error>;
