use crate::search::methods::similar_diff::compare::ChangeSimilarityComparator;
use crate::{CherryAndTarget, Commit};
use firestorm::profile_fn;
use log::debug;
use std::collections::HashSet;
use std::time::Instant;

pub fn brute_force_search(commits: &[Commit], threshold: f64) -> HashSet<CherryAndTarget> {
    profile_fn!(brute_force_search);
    let mut processed_ids = HashSet::with_capacity(commits.len());
    let mut candidates = HashSet::new();

    let start = Instant::now();
    debug!("started brute force search for {} commits", commits.len());
    let mut comparator = ChangeSimilarityComparator::new();
    for (i, first) in commits.iter().enumerate() {
        if i % 1000 == 0 {
            debug!("processed {} commits", i)
        }
        if processed_ids.contains(first.id()) {
            continue;
        }
        for (j, second) in commits[i..].iter().enumerate() {
            if first.id() == second.id() {
                continue;
            }
            if j % 1000 == 0 {
                debug!("compared {i}th commit to {} commits...", j)
            }

            if comparator.change_similarity(first.diff(), second.diff()) > threshold {
                // create a commit pair whose order depends on the commit time of both commits
                let cherry_and_target = CherryAndTarget::construct(first, second);
                candidates.insert(cherry_and_target);
            }
        }
        processed_ids.insert(first.id());
    }
    debug!("completed brute force search after {:?}", start.elapsed());
    candidates
}
