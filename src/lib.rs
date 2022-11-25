use crate::algorithms::Harvester;
use crate::git::{CommitData, LoadedRepository, RepoLocation};
use git2::{BranchType, Commit, DiffFormat, Error, Oid, Tree};
use log::debug;

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
                .filter_map(|s| {
                    // TODO: Fix unclean error handling
                    if s.0.name() != Ok(Some("origin/HEAD")) {
                        Some(s.0.get().peel_to_commit().unwrap())
                    } else {
                        None
                    }
                })
                .collect::<Vec<Commit>>();
            for head in branch_heads {
                debug!("{}", head.id());
                let mut rev_walk = repository.revwalk().unwrap();
                rev_walk.push(head.id()).unwrap();
                let mut p: Option<Tree> = Option::None;

                for id in rev_walk.map(|c| c.unwrap()) {
                    if let Ok(c) = repository.find_commit(id) {
                        let mut diff = vec![];
                        repository
                            .diff_tree_to_tree(
                                match c.parent_id(0) {
                                    Ok(pid) => {
                                        p = repository
                                            .find_commit(pid)
                                            .map(|p| p.tree().unwrap())
                                            .ok();
                                        p.as_ref()
                                    }
                                    Err(_) => None,
                                },
                                Some(&repository.find_commit(id).unwrap().tree().unwrap()),
                                None,
                            )
                            .unwrap()
                            .print(DiffFormat::Patch, |a, b, c| {
                                // diff.push(format!("{:#?}\n", a));
                                // diff.push(format!("{:#?}", b));
                                diff.push(format!(
                                    "{} {}",
                                    c.origin(),
                                    String::from_utf8(Vec::from(c.content())).unwrap()
                                ));
                                true
                            })
                            .unwrap();

                        let c = CommitData {
                            id: c.id().to_string(),
                            message: {
                                match c.message() {
                                    None => "",
                                    Some(v) => v,
                                }
                            }
                            .to_string(),
                            diff,
                            author: c.author().to_string(),
                            committer: c.committer().to_string(),
                            time: c.time(),
                        };
                        commits.push(c);
                    }
                }
            }
            debug!("{:#?}", commits[0].diff);
            harvester.harvest(&commits)
        }
    }
}
