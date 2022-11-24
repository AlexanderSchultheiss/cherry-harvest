mod util;

use git2::{Repository, Time};
use std::fmt::{Display, Formatter};
use std::path::Path;
use temp_dir::TempDir;

pub enum RepoLocation<'a> {
    FileSystem(&'a Path),
    Website(&'a str),
}

impl<'a> RepoLocation<'a> {
    pub fn to_str(&self) -> &str {
        match self {
            RepoLocation::FileSystem(path) => path
                .to_str()
                .expect("was not able to convert path to string"),
            RepoLocation::Website(url) => url,
        }
    }
}

impl<'a> Display for RepoLocation<'a> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            RepoLocation::FileSystem(_) => {
                write!(f, "\"{}\"", self.to_str())
            }
            RepoLocation::Website(url) => {
                write!(f, "\"{}\"", url)
            }
        }
    }
}

pub enum LoadedRepository {
    LocalRepo {
        path: String,
        repository: Repository,
    },
    WebRepo {
        url: String,
        repository: Repository,
        directory: TempDir,
    },
}

pub struct CommitData {
    id: String,
    message: String,
    diff: Vec<String>,
    author: String,
    committer: String,
    time: Time,
}

impl CommitData {
    pub fn new(
        id: String,
        message: String,
        diff: Vec<String>,
        author: String,
        committer: String,
        time: Time,
    ) -> Self {
        CommitData {
            id,
            message,
            diff,
            author,
            committer,
            time,
        }
    }
}
