use crate::git::LineType;
use crate::search::methods::similar_diff::compare::ChangeSimilarityComparator;
use crate::{CherryAndTarget, Commit};
use firestorm::{profile_method, profile_section};
use log::debug;
use std::collections::{HashMap, HashSet};

type Id<'a> = &'a str;
type Change = String;

#[derive(Default)]
pub struct Index<'a> {
    commit_index: HashMap<Change, HashSet<Id<'a>>>,
    change_index: HashMap<Id<'a>, HashSet<Change>>,
    commit_storage: HashMap<Id<'a>, &'a Commit>,
    threshold: f64,
}

// pub static mut COUNT: usize = 0;

impl<'a> Index<'a> {
    pub fn new(threshold: f64) -> Self {
        Self {
            threshold,
            ..Self::default()
        }
    }

    pub fn insert(&mut self, commit: &'a Commit) {
        profile_method!(insert);
        commit
            .diff()
            .hunks
            .iter()
            .flat_map(|h| {
                h.body().iter().filter_map(|l| match l.line_type() {
                    LineType::Addition
                    | LineType::Deletion
                    | LineType::AddEofnl
                    | LineType::DelEofnl => {
                        Some(format!("{} {}", l.line_type().char(), l.content().trim()))
                    }
                    _ => None,
                })
            })
            .for_each(|c| {
                // update the change_index
                let entry = self.change_index.entry(commit.id()).or_default();
                entry.insert(c.clone());

                // update the commit_index
                let entry = self.commit_index.entry(c).or_default();
                entry.insert(commit.id());
            });
        self.commit_storage.insert(commit.id(), commit);
    }

    // pub fn neighbors(&mut self, commit: &CommitData) -> HashSet<&'a str> {
    //     match self.change_index.get(commit.id()) {
    //         None => HashSet::new(),
    //         Some(changes) => {
    //             unsafe {
    //                 COUNT += changes.len();
    //             }
    //             changes
    //                 .iter()
    //                 .flat_map(|c| self.commit_index.get(c).unwrap())
    //                 .filter_map(|c| if *c != commit.id() { Some(*c) } else { None })
    //                 .collect()
    //         }
    //     }
    // }

    pub fn candidates(&self) -> HashSet<CherryAndTarget> {
        profile_method!(candidates);
        debug!("finding util among {} entries", self.commit_index.len());
        let mut candidates = HashSet::new();
        let mut comparator = ChangeSimilarityComparator::new();

        let mut pairs_to_check: HashSet<CandidatePair> = HashSet::new();
        self.commit_index.values().for_each(|neighbors| {
            profile_section!(collect_candidate_pairs);
            for n1 in neighbors {
                for n2 in neighbors {
                    pairs_to_check.insert(CandidatePair::new(n1, n2));
                }
            }
        });
        debug!("found {} unique pairs to compare", pairs_to_check.len());

        for (i, pair) in pairs_to_check.iter().enumerate() {
            profile_section!(check_candidates);
            let id_a = pair.0;
            let id_b = pair.1;
            if id_a != id_b {
                let commit_a = self.commit_storage.get(id_a).unwrap();
                let commit_b = self.commit_storage.get(id_b).unwrap();

                if comparator.change_similarity(commit_a.diff(), commit_b.diff()) > self.threshold {
                    // create a commit pair whose order depends on the commit time of both commits
                    let cherry_and_target = CherryAndTarget::construct(commit_a, commit_b);
                    candidates.insert(cherry_and_target);
                }
            }
            if i % 1000 == 0 {
                debug!(
                    "finished comparison for {}/{} pairs",
                    i,
                    pairs_to_check.len()
                );
            }
        }
        debug!(
            "found {} candidate pairs among {} possible combinations",
            candidates.len(),
            self.change_index.len() * self.change_index.len()
        );
        let percentage =
            100.0 * (1.0 - (candidates.len() as f64 / self.change_index.len().pow(2) as f64));
        debug!("reduced search by {}%", percentage);
        candidates
    }
}

#[derive(PartialEq, Eq, Hash, Debug)]
pub struct CandidatePair<'a>(pub &'a str, pub &'a str);

impl<'a> CandidatePair<'a> {
    pub fn new(c1: &'a str, c2: &'a str) -> Self {
        if c1 <= c2 {
            CandidatePair(c1, c2)
        } else {
            CandidatePair(c2, c1)
        }
    }
}

#[cfg(test)]
mod tests {
    fn init() {
        let _ = env_logger::builder().is_test(true).try_init();
    }

    #[test]
    fn index_search() {
        init();
    }
}
