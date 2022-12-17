use crate::git::CommitData;
use crate::{CommitPair, SearchMethod, SearchResult};
use hora::core::ann_index::ANNIndex;
use log::debug;
use ngrammatic::{Ngram, NgramBuilder, Pad};
use std::cmp::max;
use std::collections::{HashMap, HashSet};
use std::time::Instant;

pub const NAME: &str = "SimilarityDiffMatch";

/// SimilarityDiffMatch
#[derive(Default)]
pub struct SimilarityDiffMatch();

// TODO: This method must find at least the same cherry-picks as the exact search, otherwise it is missing cherry-picks
impl SearchMethod for SimilarityDiffMatch {
    fn search(&self, commits: &[CommitData]) -> HashSet<SearchResult> {
        debug!("retrieved a total of {} commits", commits.len());
        let start = Instant::now();

        let mut ngram_map = HashMap::<&CommitData, Ngram>::new();
        for commit in commits {
            // TODO: implement own but similar approach to improve clarity
            let ngram = NgramBuilder::new(commit.diff().diff_text())
                .arity(2)
                .pad_left(Pad::Auto)
                .pad_right(Pad::Auto)
                .finish();
            ngram_map.insert(commit, ngram);
        }
        debug!("converted all commits to ngrams in {:?}", start.elapsed());

        let start = Instant::now();

        // prepare a map of commits to f32 vectors of their diff
        // the f32 vectors are required for the ANN search
        let mut max_length = 0;
        // TODO: is there a better way to convert text to float vectors?
        let mut commit_f32_map: HashMap<&CommitData, Vec<f32>> = commits
            .iter()
            .map(|c| {
                let vec = c
                    .diff()
                    .diff_text()
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
        for (i, (commit, f32_vec)) in commit_f32_map.iter().enumerate() {
            let neighbors = index.search(f32_vec, 100);
            let neighbors = neighbors
                .into_iter()
                .map(|i| commits.get(i).unwrap())
                .collect::<Vec<&CommitData>>();
            let ngram = ngram_map.get(commit).unwrap();
            for other in neighbors {
                if *commit == other {
                    continue;
                }
                let other_ngram = ngram_map.get(other).unwrap();

                // Compare both
                if ngram.matches_with_warp(other_ngram, 1.0, 0.95).is_some() {
                    results.insert(SearchResult::new(
                        NAME.to_string(),
                        // create a commit pair whose order depends on the commit time of both commits
                        if commit.time() < other.time() {
                            // commit is older than other_commit
                            CommitPair(String::from(commit.id()), String::from(other.id()))
                        } else {
                            CommitPair(String::from(other.id()), String::from(commit.id()))
                        },
                    ));
                }
            }
            if i % 1000 == 0 {
                debug!("finished comparison for {} commits", i);
            }
        }
        debug!("found {} results in {:?} ", results.len(), start.elapsed());
        results
    }

    fn name(&self) -> &'static str {
        NAME
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
