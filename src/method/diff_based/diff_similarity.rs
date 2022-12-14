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
        let mut corpus = CorpusBuilder::new().arity(2).pad_full(Pad::Auto).finish();

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
            .enumerate()
            .flat_map(|(i, c)| {
                debug!("processing commit {}", i);
                corpus.search(c, 0.7)
            })
            .collect::<Vec<ngrammatic::SearchResult>>();
        debug!("found {} results in {:?} ", results.len(), start.elapsed());

        let mut results = HashSet::new();
        results
    }
}

/*
   fn search(&self, commits: &[CommitData]) -> HashSet<SearchResult> {
        // prepare a map of commits to f32 vectors of their diff
        // the f32 vectors are required for the ANN search
        let mut max_length = 0;
        let mut commit_f32_map: HashMap<&CommitData, Vec<f32>> = commits
            .iter()
            .map(|c| {
                let vec = c
                    .diff()
                    .to_string()
                    .as_bytes()
                    .iter()
                    .map(|b| *b as f32)
                    .take(64)
                    .collect::<Vec<f32>>();
                max_length = max(max_length, vec.len());
                (c, vec)
            })
            .collect();

        // build the index for the ANN search
        let mut index = hora::index::hnsw_idx::HNSWIndex::<f32, usize>::new(
            max_length,
            &hora::index::hnsw_params::HNSWParams::<f32>::default(),
        );
        for (i, commit) in commits.iter().enumerate() {
            // all vectors need to be padded
            let diff_as_f32: &mut Vec<f32> = commit_f32_map.get_mut(&commit).unwrap();
            while diff_as_f32.len() < max_length {
                diff_as_f32.push(0.0);
            }
            index.add(diff_as_f32, i).unwrap();
        }
        index.build(hora::core::metrics::Metric::Euclidean).unwrap();

        // search for the nearest neighbors of each commit
        let mut results = HashSet::new();
        for (commit, f32_vec) in commit_f32_map {
            let neighbors = index.search(&f32_vec, 5);
            let neighbors = neighbors
                .into_iter()
                .map(|i| commits.get(i).unwrap())
                .collect::<Vec<&CommitData>>();
            for n in neighbors {
                if commit.id() != n.id() {
                    results.insert(SearchResult::new(
                        NAME.to_string(),
                        CommitPair(commit.id().to_string(), n.id().to_string()),
                    ));
                }
            }
        }

        results
    }
*/
