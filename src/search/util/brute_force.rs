use crate::search::util::ann::CandidatePair;
use crate::Commit;
use log::debug;
use std::collections::HashSet;
use std::time::Instant;

pub fn brute_force_search(commits: &[Commit]) -> HashSet<CandidatePair> {
    let mut processed_ids = HashSet::with_capacity(commits.len());
    let mut candidates = HashSet::new();

    let start = Instant::now();
    debug!("started brute force search for {} commits", commits.len());
    for (i, first) in commits.iter().enumerate() {
        if i % 1000 == 0 {
            debug!("processed {} commits", i)
        }
        if processed_ids.contains(first.id()) {
            continue;
        }
        for second in commits[i..].iter() {
            if first.id() != second.id() && first.diff() == second.diff() {
                // create a commit pair whose order depends on the commit time of both commits
                let commit_pair = if first.time() < second.time() {
                    // commit is older than other_commit
                    CandidatePair(first.id(), second.id())
                } else {
                    CandidatePair(second.id(), first.id())
                };
                candidates.insert(commit_pair);
            }
        }
        processed_ids.insert(first.id());
    }
    debug!("completed brute force search after {:?}", start.elapsed());
    candidates
}
