use crate::search::brute_force::brute_force_search;
use crate::{CommitData, CommitPair, SearchMethod, SearchResult};
use log::debug;
use std::collections::HashSet;
use std::time::Instant;

pub const NAME: &str = "BruteForce";

/// SimilarityDiffMatch
#[derive(Default)]
pub struct BruteForceMatch();

impl SearchMethod for BruteForceMatch {
    fn search(&self, commits: &[CommitData]) -> HashSet<SearchResult> {
        let candidates = brute_force_search(commits);
        candidates
            .into_iter()
            .map(|c| {
                SearchResult::new(
                    NAME.to_string(),
                    CommitPair(c.0.to_string(), c.1.to_string()),
                )
            })
            .collect()
    }

    fn name(&self) -> &'static str {
        NAME
    }
}
