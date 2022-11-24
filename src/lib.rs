use crate::algorithms::Harvester;
use crate::git::{CommitData, LoadedRepository, RepoLocation};
use git2::{BranchType, Commit};

pub mod algorithms;
mod error;
mod git;

pub struct CherryGroup {
    cherry_ids: Vec<String>,
}

impl CherryGroup {
    fn new(cherry_ids: Vec<String>) -> Self {
        Self { cherry_ids }
    }
}

pub fn search_with<T: Harvester>(p0: &str, harvester: T) -> Vec<CherryGroup> {
    let location = RepoLocation::Website(p0);
    match git::clone_or_load(&location).unwrap() {
        LoadedRepository::LocalRepo { repository, .. }
        | LoadedRepository::WebRepo { repository, .. } => {
            let mut commits = Vec::new();
            let branch_heads = repository
                .branches(Some(BranchType::Remote))
                .unwrap()
                .map(|f| f.unwrap())
                .map(|s| s.0.get().peel_to_commit().unwrap())
                .collect::<Vec<Commit>>();
            for head in branch_heads {
                let mut rev_walk = repository.revwalk().unwrap();
                rev_walk.push(head.id()).unwrap();
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
                            diff: vec![],
                            author: c.author().to_string(),
                            committer: c.committer().to_string(),
                            time: c.time(),
                        };
                        commits.push(c);
                    }
                }
            }
            harvester.harvest(&commits)
        }
    }
}
