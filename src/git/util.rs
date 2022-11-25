use crate::error::{Error, ErrorKind};
use crate::git::LoadedRepository::{LocalRepo, RemoteRepo};
use crate::git::{CommitData, DiffData, LoadedRepository, RepoLocation};
use git2::{BranchType, Commit, Oid, Repository};
use log::{debug, error};
use temp_dir::TempDir;

/// Clone a repository into a temporary directory, or load an existing repository from the filesystem.
///
/// # Errors
/// Returns an ErrorKind::RepoCloneError, iff the given string literal was interpreted as
/// repository url and cloning the repository failed.  
///
/// Returns an ErrorKind::RepoLoadError, iff the given string literal was interpreted as path
pub fn clone_or_load(repo_location: &RepoLocation) -> Result<LoadedRepository, Error> {
    match repo_location {
        RepoLocation::Filesystem(path) => {
            debug!("loading repo from {}", repo_location);
            match Repository::open(path) {
                Ok(repo) => {
                    debug!("loaded {} successfully", repo_location);
                    Ok(LocalRepo {
                        path: String::from(repo_location.to_str()),
                        repository: repo,
                    })
                }
                Err(error) => {
                    error!("was not able to load {}; reason: {}", repo_location, error);
                    Err(Error::new(ErrorKind::RepoLoad(error)))
                }
            }
        }
        RepoLocation::Server(url) => {
            debug!("started cloning of {}", repo_location);
            // In case of repositories hosted online
            // Create a new temporary directory into which the repo can be cloned
            let temp_dir = TempDir::new().unwrap();

            // Clone the repository
            let repo = match Repository::clone(url, temp_dir.path()) {
                Ok(repo) => {
                    debug!("cloned {} successfully", repo_location);
                    repo
                }
                Err(error) => {
                    error!("was not able to clone {}; reason: {}", repo_location, error);
                    return Err(Error::new(ErrorKind::RepoClone(error)));
                }
            };

            Ok(RemoteRepo {
                url: String::from(*url),
                repository: repo,
                directory: temp_dir,
            })
        }
    }
}

/// Determine the diff of the given commit (i.e., the changes that were applied by this commit.
///
/// # Errors
/// Returns a GitDiff error, if git2 returns an error during diffing.
///
pub fn commit_diff(repository: &Repository, commit: &Commit) -> Result<DiffData, Error> {
    repository
        .diff_tree_to_tree(
            // Retrieve the parent commit and map it to an Option variant.
            // If there is no parent, the commit is considered as the root
            commit.parent(0).map(|c| c.tree().unwrap()).ok().as_ref(),
            Some(&commit.tree().unwrap()),
            None,
        )
        .map(DiffData::from)
        .map_err(|e| {
            error!("Was not able to retrieve diff for {}: {}", commit.id(), e);
            Error::new(ErrorKind::GitDiff(e))
        })
}

/// Collect the branch heads (i.e., most recent commits) of all local or remote branches.
///
/// This functions explicitly filters the HEAD, in order to not consider the current HEAD branch twice.
pub fn branch_heads(repository: &Repository, branch_type: BranchType) -> Vec<Commit> {
    repository
        .branches(Some(branch_type))
        .unwrap()
        .map(|f| f.unwrap())
        .filter_map(|s| {
            match s.0.name() {
                Ok(name) => {
                    if name != Some("origin/HEAD") && name != Some("HEAD") {
                        Some(
                            s.0.get().peel_to_commit().expect(
                                "Was not able to peel to commit while retrieving branches.",
                            ),
                        )
                    } else {
                        //
                        None
                    }
                }
                Err(err) => {
                    error!("Error while retrieving branch heads: {}", err);
                    None
                }
            }
        })
        .collect::<Vec<Commit>>()
}

/// Collect all commits in the history of the given commit, including the commit itself.
///
/// If the repo has the commit history A->B->C->D, where A is the oldest commit,
/// calling *history_for_commit(repo, C)* will return *vec![C, B, A]*.
pub fn history_for_commit(repository: &Repository, commit_id: Oid) -> Vec<CommitData> {
    debug!("{}", commit_id);
    let mut rev_walk = repository.revwalk().unwrap();
    rev_walk.push(commit_id).unwrap();

    let mut commits = vec![];
    for id in rev_walk.map(|c| c.unwrap()) {
        if let Ok(c) = repository.find_commit(id) {
            let c = CommitData {
                id: c.id().to_string(),
                message: {
                    match c.message() {
                        None => "",
                        Some(v) => v,
                    }
                }
                .to_string(),
                diff: commit_diff(repository, &c).unwrap(),
                author: c.author().to_string(),
                committer: c.committer().to_string(),
                time: c.time(),
            };
            commits.push(c);
        }
    }
    commits
}

#[cfg(test)]
mod tests {
    use super::*;
    use git2::Oid;

    fn init() {
        let _ = env_logger::builder().is_test(true).try_init();
    }

    #[test]
    fn open_local_repo() {
        init();
        use std::env;
        // We try to open this project's repository
        let path_buf = env::current_dir().unwrap();
        let location = RepoLocation::Filesystem(path_buf.as_path());
        let loaded_repo = clone_or_load(&location).unwrap();
        if let LocalRepo { path, .. } = loaded_repo {
            assert_eq!(path, location.to_str());
        }
    }

    #[test]
    fn diff_commit() {
        init();

        let expected: Vec<&str> = vec!["F diff --git a/src/git/util.rs b/src/git/util.rs\nindex bbdccd5..3af944b 100644\n--- a/src/git/util.rs\n+++ b/src/git/util.rs\n",
                                       "H @@ -64,7 +64,7 @@ pub fn commit_diff<'a, 'b>(\n", "  ) -> Result<Diff<'a>, git2::Error> {\n", 
                                       "      repository.diff_tree_to_tree(\n", 
                                       "          // Retrieve the parent commit and map it to an Option variant\n", 
                                       "-         commit.parent(0).map(|c| c.tree())?.ok().as_ref(),\n",
                                       "+         commit.parent(0).map(|c| c.tree().unwrap()).ok().as_ref(),\n",
                                       "          Some(&commit.tree().unwrap()),\n",
                                       "          None,\n", "      )\n"];

        use std::env;
        // We try to open this project's repository
        let path_buf = env::current_dir().unwrap();
        let location = RepoLocation::Filesystem(path_buf.as_path());
        let loaded_repo = clone_or_load(&location).unwrap();
        let oid = Oid::from_str("fe849e49cfe6239068ab45fa6680979c59e1bbd9").unwrap();
        if let LocalRepo { repository, .. } = loaded_repo {
            let commit = repository.find_commit(oid).unwrap();
            let diff = commit_diff(&repository, &commit).unwrap();
            assert_eq!(expected, diff.lines)
        }
    }

    #[test]
    fn clone_remote_repo() {
        init();
        let location = RepoLocation::Server("https://github.com/rust-lang/git2-rs.git");
        let loaded_repo = clone_or_load(&location).unwrap();
        if let RemoteRepo { url, .. } = loaded_repo {
            assert_eq!(url, location.to_str());
        }
    }
}
