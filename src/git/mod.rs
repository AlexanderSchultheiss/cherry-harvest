mod util;

use git2::{Diff, DiffFormat, Repository, Time};
use std::collections::HashMap;
use std::fmt::{Debug, Display, Formatter};
use std::path::{Path, PathBuf};
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

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct DiffData {
    pub hunks: Vec<Hunk>,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Hunk {
    pub old_file: Option<PathBuf>,
    pub new_file: Option<PathBuf>,
    pub lines: Vec<String>,
}

impl<'repo> From<Diff<'repo>> for DiffData {
    fn from(diff: Diff) -> Self {
        let mut hunk_map = HashMap::<String, Hunk>::new();
        diff.print(DiffFormat::Patch, |delta, hunk, diff_line| {
            match hunk {
                None => {/* Skip this delta if it does not belong to a hunk (i.e., the header line of the diff)*/}
                Some(h) => {
                    let hunk_head = String::from_utf8_lossy(h.header()).into_owned();

                    // retrieve the hunk from the map, or create it in the map if it does not exist yet
                    let hunk = hunk_map.entry(hunk_head).or_insert(Hunk {
                        old_file: delta.old_file().path().map(|f| f.to_path_buf()),
                        new_file: delta.new_file().path().map(|f| f.to_path_buf()),
                        lines: vec![],
                    });

                    // add the line to the hunk
                    hunk.lines.push(format!(
                        "{} {}",
                        diff_line.origin(),
                        String::from_utf8(Vec::from(diff_line.content()))
                            .expect("was not able to parse diff line")
                    ));
                }
            }
            true
        })
        .unwrap();
        Self {
            hunks: hunk_map.into_values().collect(),
        }
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
