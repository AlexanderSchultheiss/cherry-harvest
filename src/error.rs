use git2::Error as GError;
use std::fmt::{self, Display, Formatter};

#[derive(Debug)]
pub enum ErrorKind {
    RepoClone(GError),
    RepoLoad(GError),
    GitDiff(GError),
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
            Self::RepoLoad(error) | Self::RepoClone(error) | Self::GitDiff(error) => {
                write!(f, "{}", error)
            }
        }
    }
}

impl Display for Error {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl std::error::Error for Error {}
