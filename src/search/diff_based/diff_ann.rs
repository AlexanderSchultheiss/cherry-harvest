use crate::search::ann::Index;
use crate::{CommitData, SearchMethod, SearchResult};
use log::debug;
use std::collections::HashSet;
use std::time::Instant;

pub const NAME: &str = "ANN";

/// SimilarityDiffMatch
#[derive(Default)]
pub struct ANNMatch();

impl SearchMethod for ANNMatch {
    fn search(&self, commits: &[CommitData]) -> HashSet<SearchResult> {
        let mut index = Index::new();

        debug!("starting indexing of {} commits", commits.len());
        let start = Instant::now();
        for (i, commit) in commits.iter().enumerate() {
            index.insert(commit);
            if i % 1000 == 0 {
                debug!("finished indexing for {} commits", i);
            }
        }
        debug!("finished indexing in {:?}.", start.elapsed());

        debug!("starting neighbor search for all commits");
        let start = Instant::now();
        for (i, commit) in commits.iter().enumerate() {
            index.neighbors(commit);
            if i % 1000 == 0 {
                debug!("finished search for {} commits", i + 1);
                unsafe {
                    debug!("number of processed changes: {}", crate::search::ann::COUNT);
                    debug!("average: {}", crate::search::ann::COUNT / (i + 1));
                }
            }
        }
        debug!("finished search in {:?}.", start.elapsed());
        HashSet::new()
    }

    fn name(&self) -> &'static str {
        NAME
    }
}
