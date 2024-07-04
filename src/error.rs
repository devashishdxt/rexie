use thiserror::Error;

/// Result with `rexie::Error` as error type.
pub type Result<T> = std::result::Result<T, Error>;

/// Error type for `rexie` crate
#[derive(Debug, Error, PartialEq)]
#[non_exhaustive]
pub enum Error {
    /// Indexed DB error
    #[error("idb error")]
    IdbError(#[from] idb::Error),
    /// Couldn't abort a transaction
    #[error("couldn't abort a transaction")]
    TransactionAbortFailed,
    /// Couldn't commit a transaction
    #[error("couldn't commit a transaction")]
    TransactioncommitFailed,
}
