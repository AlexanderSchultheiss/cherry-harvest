pub mod github;
mod util;

use chrono::{DateTime, Utc};
use derivative::Derivative;
use firestorm::{profile_fn, profile_method, profile_section};
use git2::{Commit as G2Commit, Oid, Repository as G2Repository, Signature};
use git2::{Diff as G2Diff, DiffFormat, Time};
use log::info;
use octocrab::models::Repository as OctoRepo;
use octocrab::models::RepositoryId;
use std::cmp::Ordering;
use std::cmp::Ordering::Equal;
use std::collections::{HashMap, VecDeque};
use std::fmt::{Debug, Display, Formatter};
use std::path::PathBuf;
use std::time::Duration;
use temp_dir::TempDir;
use tokio::time;

pub use util::clone_or_load;
pub use util::collect_commits;

use crate::git::util::commit_diff;

/// All relevant data for a commit.
#[derive(Clone, Derivative)]
#[derivative(PartialEq, Eq, Hash)]
pub struct Commit<'repo: 'com, 'com> {
    commit_id: Oid,
    #[derivative(PartialEq = "ignore", Hash = "ignore")]
    parent_ids: Vec<Oid>,
    #[derivative(PartialEq = "ignore", Hash = "ignore")]
    repository: &'repo G2Repository,
    #[derivative(PartialEq = "ignore", Hash = "ignore")]
    commit: G2Commit<'com>,
    #[derivative(PartialEq = "ignore", Hash = "ignore")]
    diff: Option<Diff>,
}

impl<'com, 'repo> Commit<'com, 'repo> {
    fn new(repository: &'repo G2Repository, commit: G2Commit<'com>) -> Commit<'repo, 'com> {
        Self {
            commit_id: commit.id(),
            parent_ids: commit.parent_ids().collect(),
            repository,
            commit,
            diff: None,
        }
    }

    pub fn id(&self) -> Oid {
        self.commit.id()
    }

    pub fn message(&self) -> Option<&str> {
        self.commit.message()
    }

    pub fn author(&self) -> Signature {
        self.commit.author()
    }

    pub fn committer(&self) -> Signature {
        self.commit.committer()
    }

    pub fn time(&self) -> Time {
        self.commit.time()
    }

    pub fn diff(&self) -> &Diff {
        self.diff
            .as_ref()
            .expect("no diff; it must first be calculcated")
    }

    pub fn calculate_diff(&mut self) -> &Diff {
        if self.diff.is_none() {
            self.diff = Some(commit_diff(self.repository, &self.commit).unwrap());
        }
        self.diff()
    }

    pub fn parent_ids(&self) -> &[Oid] {
        &self.parent_ids
    }

    pub fn repository(&self) -> &G2Repository {
        self.repository
    }
}

#[derive(Debug, Clone)]
pub struct GitRepository {
    pub id: RepositoryId,
    pub name: String,
    pub location: RepoLocation,
    pub octorepo: Option<OctoRepo>,
}

impl GitRepository {
    pub fn new_simple(id: u64, name: String, location: RepoLocation) -> Self {
        Self {
            id: RepositoryId(id),
            name,
            location,
            octorepo: None,
        }
    }
}

impl From<OctoRepo> for GitRepository {
    fn from(octo_repo: OctoRepo) -> Self {
        GitRepository {
            id: octo_repo.id,
            name: octo_repo.name.clone(),
            location: RepoLocation::Server(octo_repo.clone_url.as_ref().unwrap().to_string()),
            octorepo: Some(octo_repo),
        }
    }
}

static mut COUNTER: u64 = 0;

/// Simplistic implementation for the purpose of easy testing
impl From<RepoLocation> for GitRepository {
    fn from(location: RepoLocation) -> Self {
        let name = location.to_string();
        let id = unsafe {
            // This is only here to make sure that no two RepoLocations have the same id.
            // Use other initialization functions, if real ids are to be used.
            COUNTER += 1;
            RepositoryId(COUNTER)
        };
        Self {
            id,
            name,
            location,
            octorepo: None,
        }
    }
}

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
/// let location = RepoLocation::Server("https://github.com/rust-lang/git2-rs.git".to_string());
/// ```
///
/// ## Specifying a local repository
/// ```
/// use std::env;
/// use cherry_harvest::RepoLocation;
/// let path_buf = env::current_dir().unwrap();
/// let location = RepoLocation::Filesystem(path_buf);
/// ```
#[derive(Debug, Clone)]
pub enum RepoLocation {
    Filesystem(PathBuf),
    Server(String),
}

impl RepoLocation {
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

impl Display for RepoLocation {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            RepoLocation::Filesystem(_) => {
                write!(f, "\"{}\"", self.to_str())
            }
            RepoLocation::Server(url) => {
                write!(f, "\"{url}\"")
            }
        }
    }
}

/// Wrapper for a repository loaded with git2.
pub enum LoadedRepository {
    LocalRepo {
        path: String,
        repository: G2Repository,
    },
    RemoteRepo {
        url: String,
        repository: G2Repository,
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
    pub fn new(content: String, line_type: LineType) -> Self {
        DiffLine { content, line_type }
    }

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
                    "unable to parse char '{value}' to LineType"
                )),
            )),
        }
    }
}

/// A CommitDiff holds all hunks with the changes that happened in a commit.
#[derive(Debug, Clone, Derivative, Eq)]
#[derivative(PartialEq, Hash)]
pub struct Diff {
    #[derivative(PartialEq = "ignore", Hash = "ignore")]
    diff_text: String,
    pub hunks: Vec<Hunk>,
}

impl Diff {
    pub fn empty() -> Self {
        Diff {
            diff_text: String::new(),
            hunks: vec![],
        }
    }

    pub fn diff_text(&self) -> &str {
        &self.diff_text
    }

    fn build_diff_text(hunks: &Vec<Hunk>) -> String {
        profile_fn!(build_diff_text);
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
                    .join("")
            );
        }
        diff_text
    }
}

impl Display for Diff {
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
        profile_method!(cmp);
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

impl<'repo> From<G2Diff<'repo>> for Diff {
    fn from(diff: G2Diff) -> Self {
        profile_fn!(from_g2diff);
        // Converts a git2::Diff to a CommitDiff by reading and converting all information relevant to us.
        let mut hunk_map = HashMap::<String, Hunk>::new();
        {
            profile_section!(diff_print);
            diff.print(DiffFormat::Patch, |delta, hunk, diff_line| {
                match hunk {
                    None => { /* Skip this delta if it does not belong to a hunk (i.e., the header line of the diff)*/ }
                    Some(h) => {
                        profile_section!(hunk_header);
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
                        drop(hunk_header);

                        // add the line to the hunk, if it is not the hunk header
                        if diff_line.origin() != 'H' {
                            profile_section!(hunk_body);
                            hunk.body.push(
                                DiffLine {
                                    content: String::from_utf8_lossy(&Vec::from(diff_line.content())).to_string(),
                                    line_type: LineType::try_from(diff_line.origin()).unwrap() }
                            );
                        }
                    }
                }
                true
            })
                .unwrap();
        }
        {
            profile_section!(collect_and_sort_hunks);
            let mut hunks: Vec<Hunk> = hunk_map.into_values().collect();
            {
                profile_section!(sort_hunks);
                hunks.sort();
            }
            Self {
                diff_text: Diff::build_diff_text(&hunks),
                hunks,
            }
        }
    }
}

/// String wrapper for representing patches extracted with IDEA IDEs
pub struct IdeaPatch(pub String);

impl From<IdeaPatch> for Diff {
    fn from(patch: IdeaPatch) -> Self {
        profile_fn!(from);
        // separator used in patches
        const SEPARATOR: &str =
            r#"==================================================================="#;
        // number of metadata lines at the start of each file diff
        const NUM_METADATA_LINES: usize = 4;

        // first, extract and trim the content
        let patch = patch.0.trim().to_string();

        // then, split the patch into its components
        let parts = patch
            .split(SEPARATOR)
            .map(|p| p.trim())
            .filter(|p| /*file diffs start with `diff`*/ p.starts_with("diff"))
            .collect::<Vec<&str>>();

        // remove metadata lines
        let mut file_diffs = vec![];
        for (i, file_diff) in parts.iter().enumerate() {
            let mut lines = file_diff
                .lines()
                .map(|l| l.to_string())
                .collect::<Vec<String>>();
            // if there there is another file diff, we have to remove metadata lines at the end of
            // the current file_diff, because they appear before the separator
            if (i + 1) < parts.len() {
                lines.truncate(lines.len() - NUM_METADATA_LINES);
            }
            file_diffs.push(lines);
        }

        // parse the textual file diffs to an instance of Diff
        let mut hunks = vec![];
        let mut hunk_headers: Vec<String> = vec![];
        let mut hunk_bodies: Vec<Vec<DiffLine>> = vec![];
        for file_diff in file_diffs {
            // split the file diff into header and hunks
            let (header, body) = file_diff.split_at(3);
            // parse the header
            let file_old = header
                .get(1)
                .unwrap()
                .split_whitespace()
                .find(|s| s.starts_with("a/"))
                .unwrap();
            let file_new = header
                .get(2)
                .unwrap()
                .split_whitespace()
                .find(|s| s.starts_with("b/"))
                .unwrap();

            // parse the hunks
            let mut body_lines = vec![];
            for line in body {
                if line.starts_with("@@ ") && line.ends_with(" @@") {
                    hunk_headers.push(line.clone());
                    if !body_lines.is_empty() {
                        hunk_bodies.push(body_lines);
                        body_lines = vec![];
                    }
                } else {
                    let line_type = LineType::try_from(line.chars().take(1).last().unwrap())
                        .unwrap_or(LineType::Context);
                    body_lines.push(DiffLine::new(line.chars().skip(1).collect(), line_type))
                }
            }
            // push the last hunk
            hunk_bodies.push(body_lines);

            // convert all hunks
            hunks.extend(
                hunk_headers
                    .into_iter()
                    .zip(hunk_bodies.into_iter())
                    .map(|(header, body)| Hunk {
                        body,
                        header,
                        old_file: Some(PathBuf::from(file_old)),
                        new_file: Some(PathBuf::from(file_new)),
                        // TODO: parse as well
                        old_start: 0,
                        new_start: 0,
                    })
                    .collect::<Vec<Hunk>>(),
            );
            hunk_headers = vec![];
            hunk_bodies = vec![];
        }
        Diff {
            diff_text: Diff::build_diff_text(&hunks),
            hunks,
        }
    }
}

// We assume that GitHub has a 60 seconds global cooldown
const DEFAULT_GLOBAL_COOLDOWN: i64 = 60;
// max requests per GLOBAL_COOLDOWN
const DEFAULT_MAX_REQUESTS: usize = 10;

struct RequestCooldown {
    queue: VecDeque<DateTime<Utc>>,
    global_cooldown: i64,
    max_requests: usize,
}

impl Default for RequestCooldown {
    fn default() -> Self {
        Self {
            queue: Default::default(),
            global_cooldown: DEFAULT_GLOBAL_COOLDOWN,
            max_requests: DEFAULT_MAX_REQUESTS,
        }
    }
}

impl RequestCooldown {
    async fn wait_for_global_cooldown(&mut self) {
        let now = Utc::now();
        let mut wait_time = None;

        // Remove previous timestamps that have cooled down
        while let Some(timestamp) = self.queue.front() {
            let seconds_passed = now.signed_duration_since(timestamp).num_seconds();
            if seconds_passed > self.global_cooldown {
                // Clean all cooled down timestamps
                self.queue.pop_front();
                continue;
            } else {
                let offset = 5;
                wait_time = Some((self.global_cooldown - seconds_passed + offset) as u64);
                break;
            }
        }

        if self.queue.len() < self.max_requests {
            // No need to wait, if we can do more requests
        } else if let Some(wait_time) = wait_time {
            // We have to wait, because we cannot do more requests
            info!("GitHub requires cooldown. Waiting for {wait_time} seconds");
            time::sleep(Duration::from_secs(wait_time)).await;
        }
        // Add a new timestamp that represents the last call
        self.queue.push_back(Utc::now());
    }
}
