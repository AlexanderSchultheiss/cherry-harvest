mod util;

use git2::{Diff, DiffFormat, Repository, Time};
use std::fmt::{Debug, Display, Formatter};
use std::path::Path;
use temp_dir::TempDir;

pub use util::branch_heads;
pub use util::clone_or_load;
pub use util::commit_diff;
pub use util::history_for_commit;

pub enum RepoLocation<'a> {
    Filesystem(&'a Path),
    Server(&'a str),
}

impl<'a> RepoLocation<'a> {
    pub fn to_str(&self) -> &str {
        match self {
            RepoLocation::Filesystem(path) => path
                .to_str()
                .expect("was not able to convert path to string"),
            RepoLocation::Server(url) => url,
        }
    }
}

impl<'a> Display for RepoLocation<'a> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            RepoLocation::Filesystem(_) => {
                write!(f, "\"{}\"", self.to_str())
            }
            RepoLocation::Server(url) => {
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
    RemoteRepo {
        url: String,
        repository: Repository,
        directory: TempDir,
    },
}

#[derive(Debug, Clone)]
pub struct DiffData {
    pub lines: Vec<String>,
}

impl<'repo> From<Diff<'repo>> for DiffData {
    fn from(diff: Diff) -> Self {
        let mut lines = vec![];
        diff.print(DiffFormat::Patch, |_, _, c| {
            lines.push(format!(
                "{} {}",
                c.origin(),
                String::from_utf8(Vec::from(c.content())).unwrap()
            ));
            true
        })
        .unwrap();
        Self { lines }
    }
}

#[derive(Debug, Clone)]
pub struct CommitData {
    pub id: String,
    pub message: String,
    pub diff: DiffData,
    pub author: String,
    pub committer: String,
    pub time: Time,
}

impl CommitData {
    pub fn new(
        id: String,
        message: String,
        diff: DiffData,
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
