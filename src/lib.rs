pub use crate::git::collect_commits;
use git2::BranchType;
use log::info;
use std::collections::HashMap;

pub mod error;
pub mod git;
pub mod search;
mod setup;

pub use git::Commit;
pub use git::Diff;
pub use git::RepoLocation;
pub use search::CherryAndTarget;
pub use search::ExactDiffMatch;
pub use search::MessageScan;
pub use search::SearchMethod;
pub use search::SearchResult;
pub use search::TraditionalLSH;

// For profiling with flame graphs to find bottlenecks
use crate::git::LoadedRepository;
pub(crate) use firestorm::{profile_fn, profile_section};

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
///         .for_each(|c| assert!(expected_commits.contains(&c.id())))
/// }
/// ```
pub fn search_with_multiple(
    repo_location: &RepoLocation,
    methods: &Vec<Box<dyn SearchMethod>>,
) -> Vec<SearchResult> {
    profile_fn!(search_with_multiple);
    info!(
        "started searching for cherry-picks in {} with {} search method(s)",
        repo_location,
        methods.len()
    );
    let mut commits = match git::clone_or_load(repo_location).unwrap() {
        LoadedRepository::LocalRepo { repository, .. } => {
            collect_commits(&repository, BranchType::Local)
        }
        LoadedRepository::RemoteRepo { repository, .. } => {
            collect_commits(&repository, BranchType::Remote)
        }
    };
    // Some commits have empty textual diffs (e.g., only changes to file modifiers)
    // We cannot consider these as cherry-picks, because no text == no information
    commits
        .retain(|commit| !commit.diff().diff_text().is_empty() && !commit.diff().hunks.is_empty());
    // Reassign to remove mutability and to convert to vector
    let commits = commits.into_iter().collect::<Vec<Commit>>();
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
///         .for_each(|c| assert!(expected_commits.contains(&c.id())))
/// }
/// ```
pub fn search_with<T: SearchMethod + 'static>(
    repo_location: &RepoLocation,
    method: T,
) -> Vec<SearchResult> {
    profile_fn!(search_with);
    search_with_multiple(repo_location, &vec![Box::new(method)])
}
