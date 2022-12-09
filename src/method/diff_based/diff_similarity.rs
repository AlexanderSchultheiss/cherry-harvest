use crate::git::{CommitData, CommitDiff};
use crate::{CommitPair, SearchMethod, SearchResult};
use log::debug;
use ngrammatic::{CorpusBuilder, Pad};
use std::collections::{HashMap, HashSet};

const NAME: &str = "SimilarityDiffMatch";
const THRESHOLD: f32 = 0.2;

/// SimilarityDiffMatch
#[derive(Default)]
pub struct SimilarityDiffMatch();

impl SearchMethod for SimilarityDiffMatch {
    fn search(&self, commits: &[CommitData]) -> HashSet<SearchResult> {
        // first simple implementation without performance improvements

        // build a corpus of all commits
        let mut corpus = CorpusBuilder::new().arity(3).pad_full(Pad::Auto).finish();
        for commit in commits {
            let neighbors = corpus.search(&commit.diff().to_string(), THRESHOLD);
            // TODO: the problem here is that we only get a diff's text as search result, but we actually want the corresponding commit
            // to achieve this, we either have to change the implementation of the ngrammatic crate, or implement our own approach
            neighbors.iter().map(|sr| &sr.text);
        }
        let results = HashSet::new();
        results
    }
}
