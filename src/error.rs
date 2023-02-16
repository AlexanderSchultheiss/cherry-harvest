use git2::Error as G2Error;
use octocrab::Error as GHError;
use std::fmt::{self, Display, Formatter};

#[derive(Debug)]
pub enum ErrorKind {
    RepoClone(G2Error),
    RepoLoad(G2Error),
    GitDiff(G2Error),
    DiffParse(String),
    ANNPreprocessing(String),
    GitHub(GHError),
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
                write!(f, "{error}")
            }
            Self::DiffParse(error) | Self::ANNPreprocessing(error) => {
                write!(f, "{error}")
            }
            Self::GitHub(error) => {
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
