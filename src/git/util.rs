use crate::error::{Error, ErrorKind};
use crate::git::LoadedRepository::{LocalRepo, RemoteRepo};
use crate::git::{Diff, LoadedRepository, RepoLocation};
use crate::Commit;
use firestorm::profile_fn;
use git2::{Branch, BranchType, Commit as G2Commit, Oid, Repository as G2Repository};
use log::{debug, error, info};
use std::collections::{HashMap, HashSet};
use std::env::temp_dir;
use std::path::Path;
use temp_dir::TempDir;

/// Clones a repository into a temporary directory, or load an existing repository from the filesystem.
///
/// # Errors
/// Returns an ErrorKind::RepoCloneError, iff the given string literal was interpreted as
/// repository url and cloning the repository failed.  
///
/// Returns an ErrorKind::RepoLoadError, iff the given string literal was interpreted as path
pub fn clone_or_load(repo_location: &RepoLocation) -> Result<LoadedRepository, Error> {
    profile_fn!(clone_or_load);
    match repo_location {
        RepoLocation::Filesystem(path) => load_local_repo(path, repo_location.to_str()),
        RepoLocation::Server(url) => clone_remote_repo(url),
    }
}

fn load_local_repo(path: &Path, path_name: &str) -> Result<LoadedRepository, Error> {
    profile_fn!(load_local_repo);
    info!("loading repo from {}", path_name);
    match G2Repository::open(path) {
        Ok(repo) => {
            debug!("loaded {} successfully", path_name);
            Ok(LocalRepo {
                path: String::from(path_name),
                repository: repo,
            })
        }
        Err(error) => {
            error!("was not able to load {}; reason: {}", path_name, error);
            Err(Error::new(ErrorKind::RepoLoad(error)))
        }
    }
}

fn clone_remote_repo(url: &str) -> Result<LoadedRepository, Error> {
    profile_fn!(clone_remote_repo);
    // In case of repositories hosted online
    // Create a new temporary directory into which the repo can be cloned
    let temp_dir = TempDir::new().unwrap();

    info!(
        "start cloning of {} into {}",
        url,
        temp_dir.path().to_str().unwrap()
    );

    // Clone the repository
    let repo = match G2Repository::clone(url, temp_dir.path()) {
        Ok(repo) => {
            debug!("cloned {} successfully", url);
            repo
        }
        Err(error) => {
            error!("was not able to clone {}; reason: {}", url, error);
            return Err(Error::new(ErrorKind::RepoClone(error)));
        }
    };

    Ok(RemoteRepo {
        url: String::from(url),
        repository: repo,
        directory: temp_dir,
    })
}

/// Collect the commits of all local or all remote branches depending on the given BranchType
pub fn collect_commits(repositories: &[LoadedRepository]) -> HashSet<Commit> {
    profile_fn!(collect_commits);
    // track commits and the repositories in which they appear. Repos are identified by their path,
    // because G2Repository does not implement Hash etc.
    let mut commits: HashMap<Commit, &G2Repository> = HashMap::new();

    // Collect the raw commits of each repo
    for (i, loaded_repository) in repositories.iter().enumerate() {
        let (repository, branch_type) = match loaded_repository {
            LocalRepo { repository, .. } => (repository, BranchType::Local),
            RemoteRepo { repository, .. } => (repository, BranchType::Remote),
        };
        let branch_heads = branch_heads(repository, branch_type);
        debug!(
            "found {} heads of {:?} branches in {i}. repository.",
            branch_heads.len(),
            branch_type
        );

        branch_heads
            .iter()
            .flat_map(|h| history_for_commit(repository, h.id()))
            .for_each(|c| {
                // hereby, we filter duplicate commits and trace each commit to the first repo it
                // was found in
                commits.entry(c).or_insert(repository);
            });

        info!("found {} commits in {i}. repository.", commits.len(),);
    }
    info!("found {} unique commits", commits.len());
    info!("converting all commits to internal representation with a diff");
    let mut unique_commits = HashSet::with_capacity(commits.len());
    for (i, (hashable_commit, _)) in commits.into_iter().enumerate() {
        if i > 0 && i % 5000 == 0 {
            info!("converted {i} commits...");
        }
        unique_commits.insert(hashable_commit);
    }
    unique_commits
}

/// Determines the diff of the given commit (i.e., the changes that were applied by this commit.
///
/// # Errors
/// Returns a GitDiff error, if git2 returns an error during diffing.
///
/// // TODO: This requires way too much time!
pub fn commit_diff(repository: &G2Repository, commit: &G2Commit) -> Result<Diff, Error> {
    profile_fn!(commit_diff);
    repository
        .diff_tree_to_tree(
            // Retrieve the parent commit and map it to an Option variant.
            // If there is no parent, the commit is considered as the root
            commit.parent(0).map(|c| c.tree().unwrap()).ok().as_ref(),
            Some(&commit.tree().unwrap()),
            None,
        )
        .map(Diff::from)
        .map_err(|e| {
            error!("Was not able to retrieve diff for {}: {}", commit.id(), e);
            Error::new(ErrorKind::GitDiff(e))
        })
}

/// Collects the branch heads (i.e., most recent commits) of all local or remote branches.
///
/// This functions explicitly filters the HEAD, in order to not consider the current HEAD branch twice.
fn branch_heads(repository: &G2Repository, branch_type: BranchType) -> Vec<G2Commit> {
    profile_fn!(branch_heads);
    repository
        .branches(Some(branch_type))
        .unwrap()
        .map(|f| f.unwrap())
        .filter_map(|(branch, _)| retrieve_regular_branch_heads(branch))
        .collect::<Vec<G2Commit>>()
}

/// Retrieve the branch's head. Omit the branch with the name _HEAD_ as this would result in duplicates.
fn retrieve_regular_branch_heads(branch: Branch) -> Option<G2Commit> {
    profile_fn!(retrieve_regular_branch_heads);
    match branch.name() {
        Ok(Some(name)) if name != "origin/HEAD" && name != "HEAD" => Some(
            branch
                .get()
                .peel_to_commit()
                .expect("Was not able to peel to commit while retrieving branches."),
        ),
        Err(err) => {
            error!("Error while retrieving branch heads: {}", err);
            None
        }
        _ => None,
    }
}

/// Collects all commits in the history of the given commit, including the commit itself.
///
/// If the repo has the commit history A->B->C->D, where A is the oldest commit,
/// calling *history_for_commit(repo, C)* will return *vec![C, B, A]*.
fn history_for_commit(repository: &G2Repository, commit_id: Oid) -> HashSet<Commit> {
    profile_fn!(history_for_commit);
    let mut processed_ids = HashSet::new();
    debug!("started collecting the history of {}", commit_id);
    let mut commits = HashSet::<Commit>::new();
    let start_commit = repository.find_commit(commit_id).unwrap();
    processed_ids.insert(start_commit.id());

    let mut parents = start_commit.parents().collect::<Vec<G2Commit>>();
    commits.insert(Commit::new(repository, start_commit));

    while !parents.is_empty() {
        let mut grandparents = vec![];
        // for each parent, add it to the vector of collected commits and collect all grandparents
        for parent in parents {
            if !processed_ids.contains(&parent.id()) {
                grandparents.extend(parent.parents());
                processed_ids.insert(parent.id());
                // we only consider non-merge commits
                if parent.parent_count() < 2 {
                    commits.insert(Commit::new(repository, parent));
                }
            }
        }
        // in the next iteration, we consider all collected grandparents
        parents = grandparents;
    }
    debug!(
        "collected {} unique commits for head {}",
        processed_ids.len(),
        commit_id
    );
    commits
}

#[cfg(test)]
mod tests {
    use git2::Oid;

    use crate::{
        git::{clone_or_load, util::commit_diff},
        LoadedRepository::{LocalRepo, RemoteRepo},
        RepoLocation,
    };

    fn init() {
        let _ = env_logger::builder().is_test(true).try_init();
    }

    #[test]
    fn open_local_repo() {
        init();
        use std::env;
        // We try to open this project's repository
        let path_buf = env::current_dir().unwrap();
        let location = RepoLocation::Filesystem(path_buf);
        let loaded_repo = clone_or_load(&location).unwrap();
        if let LocalRepo { path, .. } = loaded_repo {
            assert_eq!(path, location.to_str());
        }
    }

    #[test]
    fn diff_commit() {
        init();

        let expected: Vec<&str> = vec![
            " ) -> Result<Diff<'a>, git2::Error> {\n",
            "     repository.diff_tree_to_tree(\n",
            "         // Retrieve the parent commit and map it to an Option variant\n",
            "-        commit.parent(0).map(|c| c.tree())?.ok().as_ref(),\n",
            "+        commit.parent(0).map(|c| c.tree().unwrap()).ok().as_ref(),\n",
            "         Some(&commit.tree().unwrap()),\n",
            "         None,\n",
            "     )\n",
        ];

        use std::env;
        // We try to open this project's repository
        let path_buf = env::current_dir().unwrap();
        let location = RepoLocation::Filesystem(path_buf);
        let loaded_repo = clone_or_load(&location).unwrap();
        let oid = Oid::from_str("fe849e49cfe6239068ab45fa6680979c59e1bbd9").unwrap();
        if let LocalRepo { repository, .. } = loaded_repo {
            let commit = repository.find_commit(oid).unwrap();
            let diff = commit_diff(&repository, &commit).unwrap();
            assert_eq!(diff.hunks.len(), 1);
            assert_eq!(
                expected,
                diff.hunks[0]
                    .body
                    .iter()
                    .map(|l| l.to_string())
                    .collect::<Vec<String>>()
            )
        }
    }

    #[test]
    fn clone_remote_repo() {
        init();
        let location = RepoLocation::Server("https://github.com/rust-lang/git2-rs.git".to_string());
        let loaded_repo = clone_or_load(&location).unwrap();
        if let RemoteRepo { url, .. } = loaded_repo {
            assert_eq!(url, location.to_str());
        }
    }
}
