use std::fmt::{self, Display, Formatter};

#[derive(Debug)]
pub enum ErrorKind {
    RepoCloneError(git2::Error),
    RepoLoadError(git2::Error),
}

#[derive(Debug)]
pub struct Error(Box<ErrorKind>);

impl Error {
    pub fn new(error_kind: ErrorKind) -> Self {
        Self(Box::new(error_kind))
    }
}

impl Display for ErrorKind {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Self::RepoLoadError(error) | Self::RepoCloneError(error) => write!(f, "{}", error),
        }
    }
}

impl Display for Error {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl std::error::Error for Error {}
