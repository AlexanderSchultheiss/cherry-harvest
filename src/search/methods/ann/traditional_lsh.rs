use crate::search::methods::ann::preprocessing::{preprocess_commits, Signature};
use crate::search::methods::similar_diff::compare::ChangeSimilarityComparator;
use crate::{CherryAndTarget, Commit, SearchMethod, SearchResult};
use firestorm::profile_method;
use log::{debug, info};
use std::collections::{HashMap, HashSet};
use std::time::Instant;

pub type Band<'a> = &'a [u32];

pub fn split_signature(signature: &Signature, n_splits: usize) -> Vec<Band> {
    assert_eq!(
        signature.len() % n_splits,
        0,
        "cannot divide a signature of length {} by {n_splits}",
        signature.len()
    );

    let split_size = signature.len() / n_splits;

    let mut bands: Vec<Band> = Vec::with_capacity(n_splits);
    for band in signature.chunks(split_size) {
        bands.push(band);
    }
    bands
}

type ID = usize;

#[derive(Debug)]
pub struct TraditionalLSH {
    arity: usize,
    signature_size: usize,
    n_bands: usize,
    n_worker_threads: usize,
    threshold: f64,
}

impl TraditionalLSH {
    pub fn new(
        arity: usize,
        signature_size: usize,
        n_worker_threads: usize,
        n_bands: usize,
        threshold: f64,
    ) -> Self {
        assert_eq!(
            signature_size % n_bands,
            0,
            "a signature of length {signature_size} cannot be divided into {n_bands} bands"
        );
        Self {
            arity,
            signature_size,
            n_bands,
            n_worker_threads,
            threshold,
        }
    }

    fn build_band_maps<'sigs>(
        &self,
        signatures: &'sigs [Signature],
    ) -> Vec<HashMap<Band<'sigs>, HashSet<ID>>> {
        profile_method!(build_band_maps);
        let mut band_maps: Vec<HashMap<Band, HashSet<ID>>> = Vec::with_capacity(self.n_bands);

        // Build the band maps
        signatures
            .iter()
            .map(|signature| split_signature(signature, self.n_bands))
            .enumerate()
            .for_each(|(commit_index, bands)| {
                bands
                    .into_iter()
                    .zip(band_maps.iter_mut())
                    .for_each(|(band, map)| {
                        let entry = map.entry(band).or_insert(HashSet::new());
                        entry.insert(commit_index);
                    })
            });
        band_maps
    }

    fn collect_candidates(
        &self,
        mut band_maps: Vec<HashMap<Band, HashSet<ID>>>,
    ) -> HashSet<IdPair> {
        profile_method!(collect_candidates);
        let mut id_pairs = HashSet::new();
        band_maps
            .iter_mut()
            .flat_map(|map| {
                map.shrink_to_fit();
                map.values()
            })
            .for_each(|values| {
                for (i, id_a) in values.iter().enumerate() {
                    for id_b in values.iter().skip(i + 1) {
                        if id_a != id_b {
                            id_pairs.insert(IdPair::new(*id_a, *id_b));
                        }
                    }
                }
            });
        id_pairs
    }

    fn build_results(
        &self,
        id_pairs: HashSet<IdPair>,
        commits: &[Commit],
    ) -> HashSet<SearchResult> {
        profile_method!(build_results);
        let mut similarity_comparator = ChangeSimilarityComparator::new();
        let mut results = HashSet::new();
        for IdPair(id_a, id_b) in id_pairs.into_iter() {
            let commit_a = &commits[id_a];
            let commit_b = &commits[id_b];

            if similarity_comparator.change_similarity(commit_a, commit_b) > self.threshold {
                results.insert(SearchResult::new(
                    self.name().to_string(),
                    CherryAndTarget::construct(commit_a, commit_b),
                ));
            }
        }
        results
    }
}

impl SearchMethod for TraditionalLSH {
    fn search(&self, commits: &[Commit]) -> HashSet<SearchResult> {
        let start = Instant::now();
        info!("initialized traditional LSH approach");
        profile_method!(search_lsh);
        let signatures = preprocess_commits(
            commits,
            self.arity,
            self.signature_size,
            self.n_worker_threads,
        );
        debug!("created signatures for all commits");

        let band_maps = self.build_band_maps(&signatures);
        debug!("banded all signatures");

        // Search for pairs
        let id_pairs = self.collect_candidates(band_maps);
        debug!("collected {} candidate pairs", id_pairs.len());

        // Final similarity check
        let results = self.build_results(id_pairs, commits);
        debug!("found {} results in {:?}", results.len(), start.elapsed());
        results
    }

    fn name(&self) -> &'static str {
        "TraditionalLSH"
    }
}

#[derive(Eq, PartialEq, Hash)]
struct IdPair(ID, ID);

impl IdPair {
    fn new(id_a: ID, id_b: ID) -> Self {
        match id_a <= id_b {
            true => Self(id_a, id_b),
            false => Self(id_b, id_a),
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::search::methods::ann::traditional_lsh::{split_signature, Band};
    use std::iter::zip;

    #[test]
    fn simple_signature_split() {
        let signature = vec![1, 3, 4, 8, 23];

        let splits = split_signature(&signature, 5);
        assert_eq!(splits.len(), 5);
        splits
            .iter()
            .map(|s| s[0])
            .zip(signature.iter())
            .for_each(|(v1, v2)| assert_eq!(v1, *v2))
    }

    #[test]
    #[should_panic]
    fn invalid_signature_split() {
        let signature = vec![1, 3, 4, 8, 23];
        split_signature(&signature, 3);
    }

    #[test]
    fn single_signature_split() {
        let signature = vec![1, 3, 4, 8, 23];

        let splits = split_signature(&signature, 1);
        assert_eq!(splits.len(), 1);
        splits
            .iter()
            .flat_map(|b| b.iter())
            .zip(signature.iter())
            .for_each(|(v1, v2)| assert_eq!(v1, v2))
    }

    #[test]
    #[should_panic]
    fn zero_split() {
        let signature = vec![1, 3, 4, 8, 23];

        split_signature(&signature, 0);
    }

    #[test]
    fn signatures_are_candidates() {
        let sig_a = vec![0, 3, 2, 4, 60, 103];
        let sig_b = vec![1, 4, 2, 4, 603, 0];

        let n_splits = 3;
        let banded_a = split_signature(&sig_a, n_splits);
        let banded_b = split_signature(&sig_b, n_splits);

        assert!(candidate_check(&banded_a, &banded_b));
    }

    #[test]
    fn signatures_are_not_candidates() {
        let sig_a = vec![0, 3, 2, 4, 60, 103];
        let sig_b = vec![1, 4, 2, 5, 603, 0];

        let n_splits = 3;
        let banded_a = split_signature(&sig_a, n_splits);
        let banded_b = split_signature(&sig_b, n_splits);

        assert!(!candidate_check(&banded_a, &banded_b));
    }

    fn candidate_check(bands_a: &Vec<Band>, bands_b: &Vec<Band>) -> bool {
        zip(bands_a, bands_b).any(|(band_a, band_b)| band_a == band_b)
    }
}
