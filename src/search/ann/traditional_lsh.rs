use crate::search::ann::preprocessing::Signature;
use std::iter::zip;

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

fn candidate_check(bands_a: &Vec<Band>, bands_b: &Vec<Band>) -> bool {
    zip(bands_a, bands_b).any(|(band_a, band_b)| band_a == band_b)
}

#[cfg(test)]
mod tests {
    use crate::search::ann::traditional_lsh::{candidate_check, split_signature};

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

        let splits = split_signature(&signature, 3);
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

        let splits = split_signature(&signature, 0);
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
}
