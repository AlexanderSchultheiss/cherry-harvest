use crate::git::LoadedRepository;
use git2::{BranchType, Repository};
use log::{debug, info};
use std::collections::HashMap;

mod error;
mod git;
mod search;

pub use git::Commit;
pub use git::Diff;
pub use git::RepoLocation;
pub use search::ANNMatch;
pub use search::BruteForceMatch;
pub use search::ExactDiffMatch;
pub use search::MessageScan;
pub use search::SearchMethod;
pub use search::SimilarityDiffMatch;

#[derive(Debug, Hash, PartialEq, Eq)]
pub struct CommitPair(pub String, pub String);

// TODO: A commit can only be the target for a cherry-pick once? Or should the library return all possible source-target pairs?

impl CommitPair {
    pub fn as_vec(&self) -> Vec<&String> {
        vec![&self.0, &self.1]
    }

    pub fn into_vec(self) -> Vec<String> {
        vec![self.0, self.1]
    }
}

#[derive(Debug, Hash, PartialEq, Eq)]
pub struct SearchResult {
    search_method: String,
    commit_pair: CommitPair,
}

impl SearchResult {
    pub fn new(search_method: String, cherry_ids: CommitPair) -> Self {
        Self {
            search_method,
            commit_pair: cherry_ids,
        }
    }

    /// The SearchMethod type that was used to find this result
    pub fn search_method(&self) -> &str {
        &self.search_method
    }

    // TODO: Have references to not break connection?
    /// The commit pair of this cherry pick. Commits are identified by their id.
    pub fn commit_pair(&self) -> &CommitPair {
        &self.commit_pair
    }
}

/// Searches for cherry picks with all given search methods.
///
/// # Examples
/// TODO: Update after implementing other search methods
/// ```
/// use cherry_harvest::{MessageScan, RepoLocation};
///
/// let method = MessageScan::default();
/// // link to a test repository
/// let server = "https://github.com/AlexanderSchultheiss/cherries-one";
/// let results = cherry_harvest::search_with(&RepoLocation::Server(server), method);
/// assert_eq!(results.len(), 2);
/// let expected_commits = vec![
///     "b7d2e4b330165ae92e4442fb8ccfa067acd62d44",
///     "018a1bde4fb5e987157a6e8f07a7d378d5f19484",
///     "4e39e242712568e6f9f5b6ff113839603b722683",
///     "dd594eff3dcb36e5f4bbe47176b94f6011993c71",
/// ];
///
/// for result in results {
/// assert_eq!(result.search_method(), "MessageScan");
///     result
///         .commit_pair()
///         .as_vec()
///         .iter()
///         .for_each(|c| assert!(expected_commits.contains(&c.as_str())))
/// }
/// ```
pub fn search_with_multiple(
    repo_location: &RepoLocation,
    methods: &Vec<Box<dyn SearchMethod>>,
) -> Vec<SearchResult> {
    info!(
        "started searching for cherry-picks in {} with {} search(s)",
        repo_location,
        methods.len()
    );
    let commits = match git::clone_or_load(repo_location).unwrap() {
        LoadedRepository::LocalRepo { repository, .. } => {
            collect_commits(&repository, BranchType::Local)
        }
        LoadedRepository::RemoteRepo { repository, .. } => {
            collect_commits(&repository, BranchType::Remote)
        }
    };
    let results = methods
        .iter()
        .flat_map(|m| m.search(&commits))
        .collect::<Vec<SearchResult>>();

    info!("number of cherry-picks found by search:\n{:#?}", {
        let mut result_map = HashMap::with_capacity(methods.len());
        results
            .iter()
            .map(|r| &r.search_method)
            .for_each(|m| *result_map.entry(m).or_insert(0) += 1);
        result_map
    });

    results
}

/// Searches for cherry picks with the given search search.
///
/// # Examples
/// ```
/// use cherry_harvest::{MessageScan, RepoLocation};
///
/// // initialize the search search
/// let search = MessageScan::default();
/// // link to a test repository
/// let server = "https://github.com/AlexanderSchultheiss/cherries-one";
/// // execute the search for cherry picks
/// let results = cherry_harvest::search_with(&RepoLocation::Server(server), search);
///
/// // we expect two cherry picks
/// assert_eq!(results.len(), 2);
/// // in which a total of four commits are involved
/// let expected_commits = vec![
///     "b7d2e4b330165ae92e4442fb8ccfa067acd62d44",
///     "018a1bde4fb5e987157a6e8f07a7d378d5f19484",
///     "4e39e242712568e6f9f5b6ff113839603b722683",
///     "dd594eff3dcb36e5f4bbe47176b94f6011993c71",
/// ];
/// for result in results {
///     assert_eq!(result.search_method(), "MessageScan");
///     result
///         .commit_pair()
///         .as_vec()
///         .iter()
///         .for_each(|c| assert!(expected_commits.contains(&c.as_str())))
/// }
/// ```
pub fn search_with<T: SearchMethod + 'static>(
    repo_location: &RepoLocation,
    method: T,
) -> Vec<SearchResult> {
    search_with_multiple(repo_location, &vec![Box::new(method)])
}

/// Collect the commits of all local or all remote branches depending on the given BranchType
fn collect_commits(repository: &Repository, branch_type: BranchType) -> Vec<Commit> {
    let branch_heads = git::branch_heads(repository, branch_type);
    debug!(
        "found {} heads of {:?} branches in repository.",
        branch_heads.len(),
        branch_type
    );

    let commits: Vec<Commit> = branch_heads
        .iter()
        .flat_map(|h| git::history_for_commit(repository, h.id()))
        .collect();
    info!(
        "found {} commits in {} {:?} branches",
        commits.len(),
        branch_heads.len(),
        branch_type,
    );
    commits
}
