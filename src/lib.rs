use crate::git::{branch_heads, history_for_commit, CommitData, LoadedRepository};
use git2::{BranchType, Repository};
use log::debug;

mod error;
mod git;
mod method;

pub use git::RepoLocation;
pub use method::MessageScan;
pub use method::SearchMethod;

pub struct CherryGroup {
    pub search_method: String,
    pub cherry_ids: Vec<String>,
}

impl CherryGroup {
    fn new(search_method: String, cherry_ids: Vec<String>) -> Self {
        Self {
            search_method,
            cherry_ids,
        }
    }
}

pub fn search_with_multiple(
    repo_location: &RepoLocation,
    methods: Vec<Box<dyn SearchMethod>>,
) -> Vec<CherryGroup> {
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

pub fn search_with<T: SearchMethod + 'static>(
    repo_location: &RepoLocation,
    method: T,
) -> Vec<CherryGroup> {
    search_with_multiple(repo_location, vec![Box::new(method)])
}

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
