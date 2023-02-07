pub mod compare;

use crate::git::Commit;
use crate::{SearchMethod, SearchResult};
use firestorm::{profile_method, profile_section};
use hora::core::ann_index::ANNIndex;
use log::debug;
use ngrammatic::{Ngram, NgramBuilder, Pad};
use std::cmp::max;
use std::collections::{HashMap, HashSet};
use std::time::Instant;

pub const NAME_SIMILARITY_DIFF_MATCH: &str = "SimilarityDiffMatch";

/// SimilarityDiffMatch
#[derive(Default)]
pub struct SimilarityDiffMatch();

// TODO: This search must find at least the same cherry-picks as the exact search, otherwise it is missing cherry-picks
impl SearchMethod for SimilarityDiffMatch {
    fn search(&self, commits: &[Commit]) -> HashSet<SearchResult> {
        profile_method!(search);
        debug!("retrieved a total of {} commits", commits.len());
        let start = Instant::now();

        let mut ngram_map = HashMap::<&Commit, Ngram>::new();
        for commit in commits {
            profile_section!(build_ngram_map);
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
        let mut commit_f32_map: HashMap<&Commit, Vec<f32>> = commits
            .iter()
            .map(|c| {
                profile_section!(build_commit_f32_map);
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
        {
            profile_section!(build_index);
            for (i, commit) in commits.iter().enumerate() {
                // all vectors need to be padded
                let diff_as_f32: &mut Vec<f32> = commit_f32_map.get_mut(&commit).unwrap();
                while diff_as_f32.len() < max_length {
                    diff_as_f32.push(0.0);
                }
                index.add(diff_as_f32, i).unwrap();
            }
            index.build(hora::core::metrics::Metric::Euclidean).unwrap();
        }

        // search for the nearest neighbors of each commit
        let mut results = HashSet::new();
        let mut count = 0;
        for (i, (commit, f32_vec)) in commit_f32_map.iter().enumerate() {
            profile_section!(search_neighbors);
            let neighbors = index.search(f32_vec, 100);
            count += neighbors.len();
            let neighbors = neighbors
                .into_iter()
                .map(|i| commits.get(i).unwrap())
                .collect::<Vec<&Commit>>();
            let ngram = ngram_map.get(commit).unwrap();
            for other in neighbors {
                profile_section!(check_neighbors);
                if *commit == other {
                    continue;
                }
                let other_ngram = ngram_map.get(other).unwrap();

                // Compare both
                if ngram.matches_with_warp(other_ngram, 1.0, 0.95).is_some() {
                    results.insert(SearchResult::new(
                        NAME_SIMILARITY_DIFF_MATCH.to_string(),
                        // create a commit pair whose order depends on the commit time of both commits
                        CherryAndTarget::construct(commit, other),
                    ));
                }
            }
            if i % 1000 == 0 {
                debug!("finished comparison for {} commits", i);
                debug!("average neighbors {}", count / (i + 1));
            }
        }
        debug!("found {} results in {:?} ", results.len(), start.elapsed());
        results
    }

    fn name(&self) -> &'static str {
        NAME_SIMILARITY_DIFF_MATCH
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

use crate::search::util::brute_force::brute_force_search;
use crate::CherryAndTarget;

pub const NAME_BRUTE_FORCE: &str = "BruteForce";

/// SimilarityDiffMatch
#[derive(Default)]
pub struct BruteForceMatch();

impl SearchMethod for BruteForceMatch {
    fn search(&self, commits: &[Commit]) -> HashSet<SearchResult> {
        profile_method!(search);
        // TODO: threshold as parameter
        let candidates = brute_force_search(commits, 0.5);
        candidates
            .into_iter()
            .map(|cherry_and_target| {
                SearchResult::new(NAME_BRUTE_FORCE.to_string(), cherry_and_target)
            })
            .collect()
    }

    fn name(&self) -> &'static str {
        NAME_BRUTE_FORCE
    }
}

use crate::search::util::ann::Index;

pub const NAME_ANN: &str = "ANN";

/// SimilarityDiffMatch
#[derive(Default)]
pub struct ANNMatch();

impl SearchMethod for ANNMatch {
    fn search(&self, commits: &[Commit]) -> HashSet<SearchResult> {
        profile_method!(search);
        // TODO: threshold as parameter
        let mut index = Index::new(0.5);

        debug!("starting indexing of {} commits", commits.len());
        let start = Instant::now();
        for (i, commit) in commits.iter().enumerate() {
            profile_section!(build_index);
            index.insert(commit);
            if i % 1000 == 0 {
                debug!("finished indexing for {} commits", i);
            }
        }
        debug!("finished indexing in {:?}.", start.elapsed());

        debug!("starting neighbor search for all commits");
        let start = Instant::now();
        let candidates = index.candidates();
        debug!("finished search in {:?}.", start.elapsed());
        candidates
            .into_iter()
            .map(|cherry_and_target| SearchResult {
                search_method: NAME_ANN.to_string(),
                cherry_and_target,
            })
            .collect()
    }

    fn name(&self) -> &'static str {
        NAME_ANN
    }
}
