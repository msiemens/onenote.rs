#[cfg(feature = "backtrace")]
use std::backtrace::Backtrace;
use std::borrow::Cow;
use std::{io, string};
use thiserror::Error;

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Error, Debug)]
#[error("{kind}")]
pub struct Error {
    kind: ErrorKind,

    #[cfg(feature = "backtrace")]
    backtrace: Backtrace,
}

impl From<ErrorKind> for Error {
    #[cfg(feature = "backtrace")]
    fn from(kind: ErrorKind) -> Self {
        Error {
            kind,
            backtrace: Backtrace::capture(),
        }
    }

    #[cfg(not(feature = "backtrace"))]
    fn from(kind: ErrorKind) -> Self {
        Error { kind }
    }
}

impl From<std::io::Error> for Error {
    fn from(err: std::io::Error) -> Self {
        ErrorKind::from(err).into()
    }
}

impl From<std::string::FromUtf16Error> for Error {
    fn from(err: std::string::FromUtf16Error) -> Self {
        ErrorKind::from(err).into()
    }
}

impl From<widestring::MissingNulError<u16>> for Error {
    fn from(err: widestring::MissingNulError<u16>) -> Self {
        ErrorKind::from(err).into()
    }
}

impl From<uuid::Error> for Error {
    fn from(err: uuid::Error) -> Self {
        ErrorKind::from(err).into()
    }
}

#[derive(Error, Debug)]
pub enum ErrorKind {
    #[error("Unexpected end of file")]
    UnexpectedEOF,

    #[error("Not a table of contents file: {file}")]
    NotATocFile { file: String },

    #[error("Not a section file: {file}")]
    NotASectionFile { file: String },

    #[error("Malformed data: {0}")]
    MalformedData(Cow<'static, str>),

    #[error("Malformed OneNote data: {0}")]
    MalformedOneNoteData(Cow<'static, str>),

    #[error("Malformed OneNote file data: {0}")]
    MalformedOneNoteFileData(Cow<'static, str>),

    #[error("Malformed OneStore data: {0}")]
    MalformedOneStoreData(Cow<'static, str>),

    #[error("Malformed FSSHTTPB data: {0}")]
    MalformedFssHttpBData(Cow<'static, str>),

    #[error("Invalid UUID: {err}")]
    InvalidUuid {
        #[from]
        err: uuid::Error,
    },

    #[error("I/O failure: {err}")]
    IO {
        #[from]
        err: io::Error,
    },

    #[error("Malformed UTF-16 string: {err}")]
    Utf16Error {
        #[from]
        err: string::FromUtf16Error,
    },

    #[error("UTF-16 string is missing null terminator: {err}")]
    Utf16MissingNull {
        #[from]
        err: widestring::MissingNulError<u16>,
    },
}
