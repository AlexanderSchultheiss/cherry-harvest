mod algorithms;

use git2::{Error, Repository};
use std::fs::File;
use temp_dir::TempDir;

pub fn search_repos(repo_locations: Vec<&str>) {
    let (repositories, temp_repo_dirs) = load_repositories(repo_locations);
}

fn load_repositories(repo_locations: Vec<&str>) -> (Vec<Repository>, Vec<TempDir>) {
    let mut temporary_repo_dirs = Vec::new();
    let mut repos = Vec::new();

    // Load repositories
    for repo in repo_locations {
        if repo.starts_with("http") || repo.starts_with("www.") {
            // Create a new temporary directory into which the repo can be cloned
            let temp_dir = TempDir::new().unwrap();

            // Clone the repository
            match Repository::clone(repo, temp_dir.path()) {
                Ok(repo) => repos.push(repo),
                Err(_) => {
                    unimplemented!();
                }
            }

            // Keep the temporary directory alive until the search is done
            temporary_repo_dirs.push(temp_dir);
        } else {
            match Repository::open(repo) {
                Ok(repo) => repos.push(repo),
                Err(_) => {
                    unimplemented!()
                }
            }
        }
    }

    (repos, temporary_repo_dirs)
}

#[cfg(test)]
mod tests {
    use crate::load_repositories;
    use git2::Repository;
    use std::env;
    use temp_dir::TempDir;

    #[test]
    fn open_local_repo() {
        // We try to open this project's repository
        let repo = env::current_dir().unwrap();
        let repos = vec![repo.to_str().unwrap()];

        let (repos, temp_dirs) = load_repositories(repos);
        assert_eq!(repos.len(), 1);
        assert_eq!(temp_dirs.len(), 0);
    }

    #[test]
    fn clone_remote_repo() {
        let repos = vec!["https://github.com/rust-lang/git2-rs.git"];

        let (repos, temp_dirs) = load_repositories(repos);
        assert_eq!(repos.len(), 1);
        assert_eq!(temp_dirs.len(), 1);
    }

    #[test]
    fn load_mixed_repos() {
        let repo = env::current_dir().unwrap();
        let repos = vec![
            "https://github.com/rust-lang/git2-rs.git",
            repo.to_str().unwrap(),
        ];

        let (repos, temp_dirs) = load_repositories(repos);
        assert_eq!(repos.len(), 2);
        assert_eq!(temp_dirs.len(), 1);
    }
}
