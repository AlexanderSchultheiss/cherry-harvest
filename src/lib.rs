use crate::git::{branch_heads, history_for_commit, CommitData, LoadedRepository};
use git2::{BranchType, Repository};
use log::debug;

mod error;
mod git;
mod method;

pub use git::RepoLocation;
pub use method::MessageScan;
pub use method::SearchMethod;

pub struct CommitPair(String, String);

impl CommitPair {
    pub fn as_vec(&self) -> Vec<&String> {
        vec![&self.0, &self.1]
    }
}

pub struct SearchResult {
    pub search_method: String,
    pub commit_pair: CommitPair,
}

impl SearchResult {
    fn new(search_method: String, cherry_ids: CommitPair) -> Self {
        Self {
            search_method,
            commit_pair: cherry_ids,
        }
    }
}

/// Search for cherry picks with all given search methods.
///
/// # Examples
/// TODO: Update after implementing other search methods
/// ```
/// use cherry_harvest::{MessageScan, RepoLocation};
///
/// let method = MessageScan::default();
/// // link to a test repository
/// let server = "https://github.com/AlexanderSchultheiss/cherries-one";
/// let groups = cherry_harvest::search_with(&RepoLocation::Server(server), method);
/// assert_eq!(groups.len(), 2);
/// let expected_commits = vec![
///     "b7d2e4b330165ae92e4442fb8ccfa067acd62d44",
///     "018a1bde4fb5e987157a6e8f07a7d378d5f19484",
///     "4e39e242712568e6f9f5b6ff113839603b722683",
///     "dd594eff3dcb36e5f4bbe47176b94f6011993c71",
/// ];
///
/// for group in groups {
/// assert_eq!(group.search_method, "MessageScan");
///     group
///         .commit_pair
///         .as_vec()
///         .iter()
///         .for_each(|c| assert!(expected_commits.contains(&c.as_str())))
/// }
/// ```
pub fn search_with_multiple(
    repo_location: &RepoLocation,
    methods: Vec<Box<dyn SearchMethod>>,
) -> Vec<SearchResult> {
    let commits = match git::clone_or_load(repo_location).unwrap() {
        LoadedRepository::LocalRepo { repository, .. } => {
            collect_commits(&repository, BranchType::Local)
        }
        LoadedRepository::RemoteRepo { repository, .. } => {
            collect_commits(&repository, BranchType::Remote)
        }
    };

    methods.iter().flat_map(|m| m.search(&commits)).collect()
}

/// Search for cherry picks with the given search method.
///
/// # Examples
/// ```
/// use cherry_harvest::{MessageScan, RepoLocation};
///
/// // initialize the search method
/// let method = MessageScan::default();
/// // link to a test repository
/// let server = "https://github.com/AlexanderSchultheiss/cherries-one";
/// // execute the search for cherry picks
/// let groups = cherry_harvest::search_with(&RepoLocation::Server(server), method);
///
/// // we expect two cherry picks
/// assert_eq!(groups.len(), 2);
/// // in which a total of four commits are involved
/// let expected_commits = vec![
///     "b7d2e4b330165ae92e4442fb8ccfa067acd62d44",
///     "018a1bde4fb5e987157a6e8f07a7d378d5f19484",
///     "4e39e242712568e6f9f5b6ff113839603b722683",
///     "dd594eff3dcb36e5f4bbe47176b94f6011993c71",
/// ];
/// for group in groups {
///     assert_eq!(group.search_method, "MessageScan");
///     group
///         .commit_pair
///         .as_vec()
///         .iter()
///         .for_each(|c| assert!(expected_commits.contains(&c.as_str())))
/// }
/// ```
pub fn search_with<T: SearchMethod + 'static>(
    repo_location: &RepoLocation,
    method: T,
) -> Vec<SearchResult> {
    search_with_multiple(repo_location, vec![Box::new(method)])
}

/// Collect the commits of all local or all remote branches depending on the given BranchType
fn collect_commits(repository: &Repository, branch_type: BranchType) -> Vec<CommitData> {
    let branch_heads = branch_heads(repository, branch_type);
    debug!("Found {} {:?} branches", branch_heads.len(), branch_type,);

    let commits: Vec<CommitData> = branch_heads
        .iter()
        .flat_map(|h| history_for_commit(repository, h.id()))
        .collect();
    debug!(
        "Found {} commits in {:?} branches",
        commits.len(),
        branch_type,
    );
    commits
}
