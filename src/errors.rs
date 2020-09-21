#[cfg(feature = "backtrace")]
use std::backtrace::Backtrace;
use std::{io, string};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error("invalid UUID: {err}")]
    InvalidUuid {
        #[from]
        err: uuid::Error,

        #[cfg(feature = "backtrace")]
        backtrace: Backtrace,
    },

    #[error("io failure: {err}")]
    IO {
        #[from]
        err: io::Error,

        #[cfg(feature = "backtrace")]
        backtrace: Backtrace,
    },

    #[error("malformed UTF-16 string: {err}")]
    Utf16Error {
        #[from]
        err: string::FromUtf16Error,

        #[cfg(feature = "backtrace")]
        backtrace: Backtrace,
    },

    #[error("UTF-16 string is missing null terminator: {err}")]
    Utf16MissingNull {
        #[from]
        err: widestring::MissingNulError<u16>,

        #[cfg(feature = "backtrace")]
        backtrace: Backtrace,
    },
}

pub type Result<T> = std::result::Result<T, Error>;
