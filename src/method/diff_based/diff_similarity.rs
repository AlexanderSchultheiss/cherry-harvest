use crate::git::CommitData;
use crate::{CommitPair, SearchMethod, SearchResult};
use log::debug;
use ngrammatic::{CorpusBuilder, Pad};
use std::cmp::max;
use std::collections::{HashMap, HashSet};
use std::time::Instant;

const NAME: &str = "SimilarityDiffMatch";

/// SimilarityDiffMatch
#[derive(Default)]
pub struct SimilarityDiffMatch();

impl SearchMethod for SimilarityDiffMatch {
    fn search(&self, commits: &[CommitData]) -> HashSet<SearchResult> {
        debug!("retrieved a total of {} commits", commits.len());
        let mut corpus = CorpusBuilder::new().arity(3).pad_full(Pad::Auto).finish();

        // Build up the list of known words
        let commits: Vec<String> = commits.iter().map(|c| c.diff().to_string()).collect();
        debug!("converted all commits to strings");
        let start = Instant::now();
        for c in &commits {
            corpus.add_text(c);
        }
        debug!("added all commits to the corpus in {:?} ", start.elapsed());

        let start = Instant::now();
        let results = commits
            .iter()
            .flat_map(|c| corpus.search(c, 0.7))
            .collect::<Vec<ngrammatic::SearchResult>>();
        debug!("found {} results in {:?} ", results.len(), start.elapsed());

        let mut results = HashSet::new();
        results
    }
}
