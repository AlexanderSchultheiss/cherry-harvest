mod compare;
pub mod preprocessing;

use crate::search::methods::lsh::preprocessing::{preprocess_commits, Signature};
use crate::{CherryAndTarget, Commit, SearchMethod, SearchResult};
use firestorm::profile_method;
use log::{debug, info};
use std::collections::{HashMap, HashSet};
use std::time::Instant;

pub use compare::DiffSimilarity;

pub type Band<'a> = &'a [u32];

/// Split a given signature into n bands of size `(signature.len() / n_splits)`
///
/// # Panics
/// This functions panics if the signature cannot be split into bands of equal size (i.e., if the
/// length of the signature is not dividable by n_splits)
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

/// Implementation of traditional locality-sensitive hashing. This approach tries to find
/// commits that have highly similar diffs, but do not necessarily have to have the same diff.
///
/// This search method first converts commits into signature vectors of a given length.
/// Afterwards, the signatures are banded (i.e., split into multiple sub-vectors of equal length)
/// and the bands are hashed to individual hash maps.
///
/// The LHS approach can then identify match candidates by searching for hash conflicts among the
/// bands of different signatures. If at least one conflict occurs, the affected signatures are
/// considered match candidates.
///
/// This approach corresponds to an approximate nearest neighbor search without a strict number of
/// neighbors being searched. By searching for possible match candidates, the number of total
/// similarity comparisons can be reduced considerably. This makes it possible to consider larger
/// quantities of commits.
#[derive(Debug)]
pub struct TraditionalLSH {
    arity: usize,
    signature_size: usize,
    n_bands: usize,
    threshold: f64,
}

impl TraditionalLSH {
    /// Initialize the traditional LHS approach with the given parameters:
    /// * arity: Size of the sliding window used for the creation of the signature. This defines the
    /// size of shingles created during the shingling of a given text. A higher value
    /// will lead to more strict signatures which in turn will lead to less candidates being found.
    /// A good value to try out is `8`.
    ///
    /// * signature_size: Number of values in each signature vector. A greater number of values
    /// will improve the chance to find matching candidates, but will negatively impact the runtime.
    /// A good value to try is `100`.
    ///
    /// * band_size: LHS splits each signatures into sub-vectors (aka. bands) of this size. Smaller bands
    /// increase the chance of hash conflicts and thus lead to more candidates being found. However, this
    /// also increases the runtime. The 'signature_size' must be dividable by `band_size`. A good
    /// value to try is `5` for a signature size of `100`.
    ///
    /// * similarity_threshold: The similarity threshold must have a value in the interval `[0, 1]`.
    /// It defines the lowest value of similarity a candidate pair must have in order to be considered
    /// a real match. A good value to start is `0.75`.
    ///
    /// # Panics
    /// This function panics if the signature size cannot be divided by the band size
    /// (i.e. `signature_size % band_size != 0).
    pub fn new(
        arity: usize,
        signature_size: usize,
        band_size: usize,
        similarity_threshold: f64,
    ) -> Self {
        assert_eq!(
            signature_size % band_size,
            0,
            "a signature of length {signature_size} cannot be divided into bands of length {band_size}"
        );
        Self {
            arity,
            signature_size,
            n_bands: signature_size / band_size,
            threshold: similarity_threshold,
        }
    }

    /// Build the hash maps for the different bands. The maps are used to collect all signatures
    /// that have a hash conflict for a specific band.
    fn build_band_maps<'sigs>(
        &self,
        signatures: &'sigs [Signature],
    ) -> Vec<HashMap<Band<'sigs>, HashSet<ID>>> {
        profile_method!(build_band_maps);
        let mut band_maps: Vec<HashMap<Band, HashSet<ID>>> = vec![HashMap::default(); self.n_bands];

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
                    });
            });
        debug!("build {} of {} band maps", band_maps.len(), self.n_bands);
        band_maps
    }

    /// Collect all match candidates from the band hash maps.
    fn collect_candidates(
        &self,
        mut band_maps: Vec<HashMap<Band, HashSet<ID>>>,
    ) -> HashSet<IdPair> {
        profile_method!(collect_candidates);
        let mut id_pairs = HashSet::new();
        debug!("collecting candidates");
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

    /// Collect the final matches by comparing the similarities of match candidates
    fn build_results(
        &self,
        id_pairs: HashSet<IdPair>,
        commits: &[Commit],
    ) -> HashSet<SearchResult> {
        profile_method!(build_results);
        let mut similarity_comparator = DiffSimilarity::new();
        let mut results = HashSet::new();
        for IdPair(id_a, id_b) in id_pairs.into_iter() {
            let commit_a = &commits[id_a];
            let commit_b = &commits[id_b];
            if commit_a.id() == commit_b.id() {
                continue;
            }
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
        let signatures = preprocess_commits(commits, self.arity, self.signature_size);
        debug!(
            "created {} signatures for {} commits",
            signatures.len(),
            commits.len()
        );

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

/// Represent a pair of ids in which the ids are ordered ascending.
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
    use crate::search::methods::lsh::{split_signature, Band};
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
