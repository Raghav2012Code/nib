use thiserror::Error;

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, Error)]
pub enum Error {
    #[error("no paste {0}")]
    NotFound(String),
    #[error("ambiguous id {0}, matches: {1}")]
    Ambiguous(String, String),
    #[error("paste is {0} bytes, the limit is 65536")]
    TooLarge(usize),
    #[error("cannot unlock {0}")]
    Unlock(String),
    #[error("could not allocate an id")]
    NoId,
    #[error("{0}")]
    Message(String),
    #[error(transparent)]
    Sql(#[from] rusqlite::Error),
    #[error(transparent)]
    Io(#[from] std::io::Error),
}
