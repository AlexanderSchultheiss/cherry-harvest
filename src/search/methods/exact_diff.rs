use crate::git::{Commit, Diff};
use crate::{CommitPair, SearchMethod, SearchResult};
use log::debug;
use std::collections::{HashMap, HashSet};

pub const NAME: &str = "ExactDiffMatch";

/// ExactDiffMatch identifies cherry picks by comparing the diffs of commits.
///
/// If the diffs of two commits match exactly (considering all context lines and changed lines),
/// both commits are considered as cherry-pick and cherry. Which of the two commits is identified
/// as cherry, depends on the commits' timestamp. Here, the older commit is considered the cherry.
///
/// More precisely, ExactDiffMatch creates a HashMap of commit diffs to vectors of commits. Thereby,
/// it collects all commits whose diff have the same hash. The hash of a diff is solely determined
/// by its hunks. The hash of a hunk is determined by the hash of its body (i.e., its context lines
/// and changed lines, excluding the header line).
/// As a result, ExactDiffMatch will identify two commits as a cherry-pick, if and only if both have
/// exactly the same hunks as determined by the hunks' bodies.
///
/// If more than two commits have the same diff, multiple SearchResult instances are created by
/// considering all pairwise combinations of the commits.
/// Reminder: A cherry and its pick are determined by timestamps. Thus, there is only one SearchResult
/// for each possible commit pair.
#[derive(Default)]
pub struct ExactDiffMatch();

impl SearchMethod for ExactDiffMatch {
    fn search(&self, commits: &[Commit]) -> HashSet<SearchResult> {
        // map all commits to a hash of their diff
        let mut commit_map: HashMap<Diff, Vec<&Commit>> = HashMap::new();
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
                // consider all possible commit pairs in the vector of commits associated with the current diff
                for (index, commit) in commits.iter().enumerate() {
                    for other_commit in commits[index..].iter() {
                        if commit.id() == other_commit.id() {
                            // skip commits with the same id
                            // its the same commit in different branches, but no cherry-pick)
                            continue;
                        }

                        // create a commit pair whose order depends on the commit time of both commits
                        let commit_pair = if commit.time() < other_commit.time() {
                            // commit is older than other_commit
                            CommitPair(String::from(commit.id()), String::from(other_commit.id()))
                        } else {
                            CommitPair(String::from(other_commit.id()), String::from(commit.id()))
                        };
                        debug!("{:#?}", commit_pair);
                        debug!("{:#?} - {:#?}", commit.diff(), other_commit.diff());
                        results.push(SearchResult::new(NAME.to_string(), commit_pair));
                    }
                }
                results
            })
            .collect()
    }

    fn name(&self) -> &'static str {
        NAME
    }
}
