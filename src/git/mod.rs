mod util;

use derivative::Derivative;
use git2::{Diff, DiffFormat, Repository, Time};
use std::cmp::Ordering;
use std::cmp::Ordering::Equal;
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

#[derive(Debug, Clone, Derivative, Eq)]
#[derivative(PartialEq, Hash)]
pub struct DiffData {
    pub hunks: Vec<Hunk>,
}

#[derive(Debug, Clone, Derivative)]
#[derivative(Hash)]
pub struct Hunk {
    #[derivative(Hash = "ignore")]
    pub header: String,
    pub old_file: Option<PathBuf>,
    pub new_file: Option<PathBuf>,
    pub lines: Vec<String>,
    pub old_start: u32,
    pub new_start: u32,
}

impl PartialEq<Self> for Hunk {
    fn eq(&self, other: &Self) -> bool {
        self.old_file == other.old_file
            && self.new_file == other.new_file
            && self.lines == other.lines
    }
}

impl PartialOrd for Hunk {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Eq for Hunk {}

impl Ord for Hunk {
    fn cmp(&self, other: &Self) -> Ordering {
        // try to order hunks with precedence of old_file over new_file over start line
        let old_file_ordering = self.old_file.cmp(&other.old_file);
        let new_file_ordering = self.new_file.cmp(&other.new_file);
        let old_start_ordering = self.old_start.cmp(&other.old_start);
        let new_start_ordering = self.new_start.cmp(&other.new_start);

        // first, try ordering by the old file
        match old_file_ordering {
            // if there is no ordering for the old file, or if the old file is the same, order by the new file
            Equal => match new_file_ordering {
                // if there is no ordering for the new file, of if the new file is the same, order by the start line
                Equal => match old_start_ordering {
                    Equal => new_start_ordering,
                    ordering => ordering,
                },
                // if there is an ordering of the new file, return it
                ordering => ordering,
            },
            // if there is an ordering for the old file, return it
            ordering => ordering,
        }
    }
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
                    let hunk = hunk_map.entry(hunk_head.clone()).or_insert(Hunk {
                        header: hunk_head,
                        old_file: delta.old_file().path().map(|f| f.to_path_buf()),
                        new_file: delta.new_file().path().map(|f| f.to_path_buf()),
                        lines: vec![],
                        old_start: h.old_start(),
                        new_start: h.new_start(),
                    });

                    // add the line to the hunk, if it is not the hunk header
                    if diff_line.origin() != 'H' {
                        hunk.lines.push(format!(
                            "{} {}",
                            diff_line.origin(),
                            String::from_utf8(Vec::from(diff_line.content()))
                                .expect("was not able to parse diff line")
                        ));
                    }
                }
            }
            true
        })
        .unwrap();
        let mut hunks: Vec<Hunk> = hunk_map.into_values().collect();
        hunks.sort();
        Self { hunks }
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
