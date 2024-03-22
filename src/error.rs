use git2::Error as G2Error;
use octocrab::Error as GHError;
use serde_yaml::Error as SerdeError;
use std::fmt::{self, Display, Formatter};
use std::io::Error as IOError;

#[derive(Debug)]
pub enum ErrorKind {
    RepoClone(G2Error),
    RepoLoad(G2Error),
    GitDiff(G2Error),
    DiffParse(String),
    ANNPreprocessing(String),
    GitHub(GHError),
    Serde(SerdeError),
    IO(IOError),
}

#[derive(Debug)]
pub struct Error(ErrorKind);

impl Error {
    pub fn new(error_kind: ErrorKind) -> Self {
        Self(error_kind)
    }
}

impl Display for ErrorKind {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Self::RepoLoad(error) | Self::RepoClone(error) | Self::GitDiff(error) => {
                write!(f, "{error}")
            }
            Self::DiffParse(error) | Self::ANNPreprocessing(error) => {
                write!(f, "{error}")
            }
            Self::GitHub(error) => {
                write!(f, "{error}")
            }
            ErrorKind::Serde(error) => {
                write!(f, "{error}")
            }
            ErrorKind::IO(error) => {
                write!(f, "{error}")
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

impl From<SerdeError> for Error {
    fn from(error: serde_yaml::Error) -> Self {
        Self(ErrorKind::Serde(error))
    }
}

impl From<IOError> for Error {
    fn from(error: std::io::Error) -> Self {
        Self(ErrorKind::IO(error))
    }
}
