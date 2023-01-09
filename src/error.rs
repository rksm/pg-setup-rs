#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("Unable to parse postgres URL: {0}")]
    PgUrlParseError(String),
    #[error("Error running process {0}")]
    ProcessError(#[from] std::io::Error),
    #[error("Sqlx error: {0}")]
    SqlxError(#[from] sqlx::Error),
    #[error("{0}")]
    Other(String),
}

// type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;
pub(crate) type Result<T> = std::result::Result<T, Error>;
