use crate::git::{CommitData, CommitDiff};
use crate::{CommitPair, SearchMethod, SearchResult};
use log::debug;
use std::collections::{HashMap, HashSet};

const NAME: &str = "ExactDiffMatch";

#[derive(Default)]
// TODO: documentation
pub struct ExactDiffMatch();

impl SearchMethod for ExactDiffMatch {
    fn search(&self, commits: &[CommitData]) -> HashSet<SearchResult> {
        // map all commits to a hash of their diff
        let mut commit_map: HashMap<CommitDiff, Vec<&CommitData>> = HashMap::new();
        commits.iter().for_each(|commit| {
            commit_map
                .entry(commit.diff().clone())
                .or_default()
                .push(commit);
        });

        // then, return results for all entries with more than one commit mapped to them
        commit_map
            .iter()
            .filter_map(|(_, commits)| {
                if commits.len() > 1 {
                    Some(commits)
                } else {
                    None
                }
            })
            .flat_map(|commits| {
                let mut results = vec![];
                for (index, commit) in commits.iter().enumerate() {
                    for other_commit in commits[index..].iter() {
                        if commit.id() != other_commit.id() {
                            let commit_pair = if commit.time() < other_commit.time() {
                                // commit is older than the other_commit
                                CommitPair(
                                    String::from(commit.id()),
                                    String::from(other_commit.id()),
                                )
                            } else {
                                CommitPair(
                                    String::from(other_commit.id()),
                                    String::from(commit.id()),
                                )
                            };
                            debug!("{:#?}", commit_pair);
                            debug!("{:#?} - {:#?}", commit.diff(), other_commit.diff());
                            results.push(SearchResult::new(NAME.to_string(), commit_pair));
                        }
                    }
                }
                results
            })
            .collect()
    }
}
