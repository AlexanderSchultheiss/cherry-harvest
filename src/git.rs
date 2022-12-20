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

/// The location of a git repository. A repository can either be located locally in the file system or
/// online on a server.
///
/// A repository in the file system is located via the path to its root directory.
///
/// A repository on a server is located via the *https* clone link.
///
/// # Examples
/// ## Specifying a remote repository
/// ```
/// use cherry_harvest::RepoLocation;
/// let location = RepoLocation::Server("https://github.com/rust-lang/git2-rs.git");
/// ```
///
/// ## Specifying a local repository
/// ```
/// use std::env;
/// use cherry_harvest::RepoLocation;
/// let path_buf = env::current_dir().unwrap();
/// let location = RepoLocation::Filesystem(path_buf.as_path());
/// ```
pub enum RepoLocation<'a> {
    Filesystem(&'a Path),
    Server(&'a str),
}

impl<'a> RepoLocation<'a> {
    /// Creates a string slice of either the path or the url to the repository, depending on the
    /// RepoLocation variant.
    fn to_str(&self) -> &str {
        match self {
            RepoLocation::Filesystem(path) => {
                path.to_str().expect("was not able to convert path to str")
            }
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

/// Wrapper for a repository loaded with git2.
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

/// Represents a single line in a Diff
#[derive(Debug, Eq, PartialEq, Hash, Clone)]
pub struct DiffLine {
    content: String,
    line_type: LineType,
}

impl Display for DiffLine {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}{}", self.line_type.char(), self.content)
    }
}

impl DiffLine {
    pub fn content(&self) -> &str {
        &self.content
    }
    pub fn line_type(&self) -> LineType {
        self.line_type
    }
}

/// Type of line in a diff.
/// ```text
/// ' '  Line context
/// '+'  Line addition
/// '-'  Line deletion
/// '='  Context (End of file)
/// '>'  Add (End of file)
/// '<'  Remove (End of file)
/// 'F'  File header
/// 'H'  Hunk header
/// 'B'  Line binary
/// ```
#[derive(Debug, Eq, PartialEq, Hash, Copy, Clone)]
pub enum LineType {
    Context,
    Addition,
    Deletion,
    ContextEofnl,
    AddEofnl,
    DelEofnl,
    FileHdr,
    HunkHdr,
    Binary,
}

impl LineType {
    pub fn char(&self) -> char {
        match self {
            LineType::Context => ' ',
            LineType::Addition => '+',
            LineType::Deletion => '-',
            LineType::ContextEofnl => '=',
            LineType::AddEofnl => '>',
            LineType::DelEofnl => '<',
            LineType::FileHdr => 'F',
            LineType::HunkHdr => 'H',
            LineType::Binary => 'B',
        }
    }
}

impl TryFrom<char> for LineType {
    type Error = crate::error::Error;

    fn try_from(value: char) -> Result<Self, Self::Error> {
        match value {
            ' ' => Ok(Self::Context),
            '+' => Ok(Self::Addition),
            '-' => Ok(Self::Deletion),
            '=' => Ok(Self::ContextEofnl),
            '>' => Ok(Self::AddEofnl),
            '<' => Ok(Self::DelEofnl),
            'F' => Ok(Self::FileHdr),
            'H' => Ok(Self::HunkHdr),
            'B' => Ok(Self::Binary),
            _ => Err(crate::error::Error::new(
                crate::error::ErrorKind::DiffParse(format!(
                    "unable to parse char '{}' to LineType",
                    value
                )),
            )),
        }
    }
}

/// A CommitDiff holds all hunks with the changes that happened in a commit.
#[derive(Debug, Clone, Derivative, Eq)]
#[derivative(PartialEq, Hash)]
pub struct CommitDiff {
    #[derivative(PartialEq = "ignore", Hash = "ignore")]
    diff_text: String,
    pub hunks: Vec<Hunk>,
}

impl CommitDiff {
    pub fn empty() -> Self {
        CommitDiff {
            diff_text: String::new(),
            hunks: vec![],
        }
    }

    pub fn diff_text(&self) -> &str {
        &self.diff_text
    }

    fn build_diff_text(hunks: &Vec<Hunk>) -> String {
        let mut diff_text = String::new();
        for hunk in hunks {
            diff_text += &format!(
                "--- {}\n+++ {}\n{}\n{}\n",
                hunk.old_file
                    .as_ref()
                    .map_or("None", |pb| pb.to_str().unwrap_or("None")),
                hunk.new_file
                    .as_ref()
                    .map_or("None", |pb| pb.to_str().unwrap_or("None")),
                hunk.header,
                hunk.body
                    .iter()
                    .map(|l| l.to_string())
                    .collect::<Vec<String>>()
                    .join("\n")
            );
        }
        diff_text
    }
}

impl Display for CommitDiff {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.diff_text)
    }
}

/// A Hunk groups changes to a file that happened in a single commit.
///
/// Changes are grouped by location and a single hunk contains all change and context lines that are
/// directly adjacent to each other in a file.
#[derive(Debug, Clone, Derivative)]
#[derivative(Hash)]
pub struct Hunk {
    // The hash of a diff is only identified by its body
    body: Vec<DiffLine>,
    #[derivative(Hash = "ignore")]
    header: String,
    #[derivative(Hash = "ignore")]
    old_file: Option<PathBuf>,
    #[derivative(Hash = "ignore")]
    new_file: Option<PathBuf>,
    #[derivative(Hash = "ignore")]
    old_start: u32,
    #[derivative(Hash = "ignore")]
    new_start: u32,
}

impl Hunk {
    /// The header line of a hunk. This line contains information about the hunk's location and size
    pub fn header(&self) -> &str {
        &self.header
    }
    /// The old file to which diff was applied (i.e., the previous version of the file).
    /// None if the file did not exist yet.
    pub fn old_file(&self) -> &Option<PathBuf> {
        &self.old_file
    }
    /// The new file to which diff was applied (i.e., the current version of the file (current with respect to diffed commit)).
    /// None if the file does not exist anymore.
    pub fn new_file(&self) -> &Option<PathBuf> {
        &self.new_file
    }
    /// The lines belonging to the body of this hunk including context lines and changed lines
    pub fn body(&self) -> &Vec<DiffLine> {
        &self.body
    }
    /// The start line in the previous version
    pub fn old_start(&self) -> u32 {
        self.old_start
    }
    /// The start line in the current version
    pub fn new_start(&self) -> u32 {
        self.new_start
    }
}

impl PartialEq<Self> for Hunk {
    fn eq(&self, other: &Self) -> bool {
        self.old_file == other.old_file
            && self.new_file == other.new_file
            && self.body == other.body
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

impl<'repo> From<Diff<'repo>> for CommitDiff {
    fn from(diff: Diff) -> Self {
        // Converts a git2::Diff to a CommitDiff by reading and converting all information relevant to us.
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
                        body: vec![],
                        old_start: h.old_start(),
                        new_start: h.new_start(),
                    });
                    // add the line to the hunk, if it is not the hunk header
                    if diff_line.origin() != 'H' {
                        hunk.body.push(DiffLine { content: String::from_utf8_lossy(&Vec::from(diff_line.content())).to_string(), line_type: LineType::try_from(diff_line.origin()).unwrap() }
                        );
                    }
                }
            }
            true
        })
            .unwrap();
        let mut hunks: Vec<Hunk> = hunk_map.into_values().collect();
        hunks.sort();
        Self {
            diff_text: CommitDiff::build_diff_text(&hunks),
            hunks,
        }
    }
}

/// All relevant data for a commit.
#[derive(Debug, Clone, Derivative)]
#[derivative(PartialEq, Eq, Hash)]
pub struct CommitData {
    id: String,
    message: String,
    diff: CommitDiff,
    author: String,
    committer: String,
    #[derivative(PartialEq = "ignore", Hash = "ignore")]
    time: Time,
}

impl CommitData {
    /// Initializes a CommitData instance with the given values
    pub fn new(
        id: String,
        message: String,
        diff: CommitDiff,
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

    /// The commit hash, aka. revision number
    pub fn id(&self) -> &str {
        &self.id
    }

    /// The commit message
    pub fn message(&self) -> &str {
        &self.message
    }

    /// The diff of the commit to its first parent
    pub fn diff(&self) -> &CommitDiff {
        &self.diff
    }

    /// The author of the commit
    pub fn author(&self) -> &str {
        &self.author
    }

    /// The committer of the commit
    pub fn committer(&self) -> &str {
        &self.committer
    }

    /// The timestamp of the commit
    pub fn time(&self) -> Time {
        self.time
    }
}
