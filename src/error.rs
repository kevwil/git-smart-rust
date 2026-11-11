use std::{fmt, io};

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug)]
pub enum Error {
    /// A problem we can explain to the user in one message.
    Failed(String),
    /// A git command failed; carries its (already formatted) output.
    Git(String),
    Io(io::Error),
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::Failed(msg) | Error::Git(msg) => f.write_str(msg),
            Error::Io(e) => e.fmt(f),
        }
    }
}

impl std::error::Error for Error {}

impl From<io::Error> for Error {
    fn from(e: io::Error) -> Self {
        Error::Io(e)
    }
}

/// Shorthand for building an `Error::Failed`.
pub fn failed(msg: impl Into<String>) -> Error {
    Error::Failed(msg.into())
}
