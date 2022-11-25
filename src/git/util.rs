use crate::error::{Error, ErrorKind};
use crate::git::LoadedRepository::{LocalRepo, WebRepo};
use crate::git::{LoadedRepository, RepoLocation};
use git2::{Commit, Diff, Repository, Tree};
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
        RepoLocation::FileSystem(path) => {
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
                    Err(Error::new(ErrorKind::RepoLoadError(error)))
                }
            }
        }
        RepoLocation::Website(url) => {
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
                    return Err(Error::new(ErrorKind::RepoCloneError(error)));
                }
            };

            Ok(WebRepo {
                url: String::from(*url),
                repository: repo,
                directory: temp_dir,
            })
        }
    }
}

/// Determine the diff of the given commit (i.e., the changes that were applied by this commit.
pub fn commit_diff<'a, 'b>(
    repository: &'a Repository,
    commit: &'b Commit,
) -> Result<Diff<'a>, git2::Error> {
    repository.diff_tree_to_tree(
        // Retrieve the parent commit and map it to an Option variant
        commit.parent(0).map(|c| c.tree().unwrap()).ok().as_ref(),
        Some(&commit.tree().unwrap()),
        None,
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    fn init() {
        let _ = env_logger::builder().is_test(true).try_init();
    }

    #[test]
    fn open_local_repo() {
        init();
        use std::env;
        // We try to open this project's repository
        let path_buf = env::current_dir().unwrap();
        let location = RepoLocation::FileSystem(path_buf.as_path());
        let loaded_repo = clone_or_load(&location).unwrap();
        if let LocalRepo { path, .. } = loaded_repo {
            assert_eq!(path, location.to_str());
        }
    }

    #[test]
    fn clone_remote_repo() {
        init();
        let location = RepoLocation::Website("https://github.com/rust-lang/git2-rs.git");
        let loaded_repo = clone_or_load(&location).unwrap();
        if let WebRepo { url, .. } = loaded_repo {
            assert_eq!(url, location.to_str());
        }
    }
}
