pub use crate::git::collect_commits;
use log::{error, info};
use sampling::Sample;
use std::collections::HashMap;
use std::collections::HashSet;
use std::fs;
use std::fs::File;
use std::io::Write;
use std::path::Path;

pub mod error;
pub mod git;
pub mod sampling;
pub mod search;

pub use error::Error;
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
use crate::git::{GitRepository, LoadedRepository};
pub(crate) use firestorm::{profile_fn, profile_section};

pub type Result<T> = std::result::Result<T, Error>;

// TODO: Check out GitHub torrent for science

/// Searches for cherry picks with all given search methods.
///
/// # Examples
/// TODO: Update after implementing other search methods
/// ```
/// use cherry_harvest::{MessageScan, RepoLocation};
/// use cherry_harvest::git::GitRepository;
///
/// let method = MessageScan::default();
/// // link to a test repository
/// let server = "https://github.com/AlexanderSchultheiss/cherries-one".to_string();
/// let runtime = tokio::runtime::Runtime::new().unwrap();
/// let results = runtime.block_on(
///     cherry_harvest::search_with(&[&GitRepository::from(RepoLocation::Server(server))], method)
/// ).unwrap().1;
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
pub async fn search_with_multiple(
    repos: &[&GitRepository],
    methods: &[Box<dyn SearchMethod>],
) -> Result<(TotalCommitsCount, Vec<SearchResult>)> {
    let repo_locations: Vec<&RepoLocation> = repos.iter().map(|r| &r.location).collect();
    profile_fn!(search_with_multiple);
    info!(
        "started searching for cherry-picks in {} projects with {} search method(s)",
        repo_locations.len(),
        methods.len()
    );
    // TODO: Collect commits in parallel
    let mut loaded_repos: Vec<LoadedRepository> = Vec::new();
    for repo_location in repo_locations.iter() {
        match git::clone_or_load(repo_location).await {
            Ok(repo) => loaded_repos.push(repo),
            Err(error) => {
                error!("was not able to clone or load repository: {error}");
                return Err(error);
            }
        }
    }
    let commits = collect_commits(&loaded_repos);
    // Some commits have empty textual diffs (e.g., only changes to file modifiers)
    // We cannot consider these as cherry-picks, because no text == no information
    // TODO: Migrate to better location
    // info!("filtering commits with empty textual diffs");
    // commits.retain(|commit| {
    //     !commit.calculate_diff().diff_text().is_empty() && !commit.calculate_diff().hunks.is_empty()
    // });
    info!(
        "searching among {} unique commits from {} repositories",
        commits.len(),
        repos.len()
    );
    // Reassign to convert to vector
    let mut commits = commits.into_iter().collect::<Vec<Commit>>();
    {
        profile_section!(map_results);
        let results = methods
            .iter()
            .flat_map(|m| m.search(&mut commits))
            .collect::<Vec<SearchResult>>();

        info!(
            "number of cherry-picks found in {} repositories by search:\n{:#?}",
            repos.len(),
            {
                let mut result_map = HashMap::with_capacity(methods.len());
                results
                    .iter()
                    .map(|r| r.search_method())
                    .for_each(|m| *result_map.entry(m).or_insert(0) += 1);
                result_map
            }
        );

        Ok((commits.len(), results))
    }
}

pub type TotalCommitsCount = usize;

/// Searches for cherry picks with the given search search.
///
/// # Examples
/// ```
/// use cherry_harvest::{MessageScan, RepoLocation};
/// use cherry_harvest::git::GitRepository;
///
/// // initialize the search search
/// let search = MessageScan::default();
/// // link to a test repository
/// let server = "https://github.com/AlexanderSchultheiss/cherries-one".to_string();
/// // execute the search for cherry picks
/// let runtime = tokio::runtime::Runtime::new().unwrap();
/// let results = runtime.block_on(
///     cherry_harvest::search_with(&[&GitRepository::from(RepoLocation::Server(server))], search)
/// ).unwrap().1;
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
pub async fn search_with<T: SearchMethod + 'static>(
    repos: &[&GitRepository],
    method: T,
) -> Result<(TotalCommitsCount, Vec<SearchResult>)> {
    profile_fn!(search_with);
    search_with_multiple(repos, &[Box::new(method)]).await
}

pub fn save_repo_sample<P: AsRef<Path>>(path: P, sample: &Sample) -> Result<()> {
    let sample = serde_yaml::to_string(&sample)?;
    fs::write(path, sample)?;
    Ok(())
}

pub fn load_repo_sample<P: AsRef<Path>>(path: P) -> Result<Sample> {
    let file = fs::File::open(path)?;
    Ok(serde_yaml::from_reader(file)?)
}

pub type RepoName = String;

pub struct HarvestTracker {
    success_tracking_file: File,
    error_tracking_file: File,
    harvested_repos: HashSet<RepoName>,
    failed_repos: HashSet<RepoName>,
}

impl HarvestTracker {
    fn load_repo_list<P: AsRef<Path>>(path_to_file: P) -> Result<(HashSet<RepoName>, File)> {
        Ok(if Path::exists(path_to_file.as_ref()) {
            let repos = serde_yaml::from_str(&fs::read_to_string(&path_to_file)?)?;
            let file = File::options().append(true).open(&path_to_file)?;
            (repos, file)
        } else {
            (HashSet::new(), File::create_new(path_to_file)?)
        })
    }

    pub fn load_harvest_tracker<P: AsRef<Path>>(
        path_to_success_tracking_file: P,
        path_to_error_tracking_file: P,
    ) -> Result<HarvestTracker> {
        let (harvested_repos, success_tracking_file) =
            HarvestTracker::load_repo_list(path_to_success_tracking_file)?;
        let (failed_repos, error_tracking_file) =
            HarvestTracker::load_repo_list(path_to_error_tracking_file)?;

        Ok(HarvestTracker {
            success_tracking_file,
            error_tracking_file,
            harvested_repos,

            failed_repos,
        })
    }

    pub fn contains(&self, repo: &RepoName) -> bool {
        self.harvested_repos.contains(repo)
    }

    pub fn add_success(&mut self, repo: RepoName) -> Result<()> {
        let repo = format!("- {repo}\n");
        self.success_tracking_file.write_all(repo.as_bytes())?;
        self.harvested_repos.insert(repo);
        Ok(())
    }

    pub fn add_error(&mut self, repo: RepoName) -> Result<()> {
        let repo = format!("- {repo}\n");
        self.error_tracking_file.write_all(repo.as_bytes())?;
        self.failed_repos.insert(repo);
        Ok(())
    }
}
