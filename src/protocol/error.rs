//! errors during protocol execution

use std::fmt::Debug;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum Error {
    IOError(std::io::ErrorKind),
    MPCError(MPCErrorKind),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
#[non_exhaustive]

pub enum MPCErrorKind {
    InsufficientShares,
}

impl From<std::io::Error> for Error {
    fn from(err: std::io::Error) -> Self {
        Error::IOError(err.kind())
    }
}

pub type Result<T> = std::result::Result<T, Error>;
