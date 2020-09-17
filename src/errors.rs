use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error("invalid UUID: {0}")]
    InvalidUuid(#[from] uuid::Error),
}

pub type Result<T> = std::result::Result<T, Error>;
