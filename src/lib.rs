use crate::git::LoadedRepository;
use git2::{BranchType, Repository};
use log::{debug, info};
use std::collections::HashMap;

pub mod error;
pub mod git;
pub mod search;

pub use git::Commit;
pub use git::Diff;
pub use git::RepoLocation;
pub use search::ANNMatch;
pub use search::BruteForceMatch;
pub use search::CherryAndTarget;
pub use search::ExactDiffMatch;
pub use search::HNSWSearch;
pub use search::MessageScan;
pub use search::SearchMethod;
pub use search::SearchResult;
pub use search::SimilarityDiffMatch;

// For profiling with flame graphs to find bottlenecks
pub(crate) use firestorm::{profile_fn, profile_method, profile_section};

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
    profile_fn!(search_with_multiple);
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
    {
        profile_section!(map_results);
        let results = methods
            .iter()
            .flat_map(|m| m.search(&commits))
            .collect::<Vec<SearchResult>>();

        info!("number of cherry-picks found by search:\n{:#?}", {
            let mut result_map = HashMap::with_capacity(methods.len());
            results
                .iter()
                .map(|r| r.search_method())
                .for_each(|m| *result_map.entry(m).or_insert(0) += 1);
            result_map
        });

        results
    }
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
    profile_fn!(search_with);
    search_with_multiple(repo_location, &vec![Box::new(method)])
}

/// Collect the commits of all local or all remote branches depending on the given BranchType
pub fn collect_commits(repository: &Repository, branch_type: BranchType) -> Vec<Commit> {
    profile_fn!(collect_commits);
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
