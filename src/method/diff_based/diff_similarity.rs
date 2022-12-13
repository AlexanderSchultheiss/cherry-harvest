use crate::git::CommitData;
use crate::{CommitPair, SearchMethod, SearchResult};
use hora;
use hora::core::ann_index::ANNIndex;
use std::cmp::max;
use std::collections::{HashMap, HashSet};

const NAME: &str = "SimilarityDiffMatch";

/// SimilarityDiffMatch
#[derive(Default)]
pub struct SimilarityDiffMatch();

impl SearchMethod for SimilarityDiffMatch {
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
        let mut index = hora::index::pq_idx::PQIndex::<f32, usize>::new(
            max_length,
            &hora::index::pq_params::PQParams::default(),
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
            let neighbors = index.search(&f32_vec, 10);
            let neighbors = neighbors
                .into_iter()
                .map(|i| commits.get(i).unwrap())
                .collect::<Vec<&CommitData>>();
            for n in neighbors {
                results.insert(SearchResult::new(
                    NAME.to_string(),
                    CommitPair(commit.id().to_string(), n.id().to_string()),
                ));
            }
        }

        results
    }
}
