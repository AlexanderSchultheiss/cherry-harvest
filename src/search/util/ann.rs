use crate::git::LineType;
use crate::CommitData;
use log::debug;
use std::collections::{HashMap, HashSet};
use std::hash::Hash;

type Id<'a> = &'a str;
type Change = String;

#[derive(Default)]
pub struct Index<'a> {
    commit_index: HashMap<Change, HashSet<Id<'a>>>,
    change_index: HashMap<Id<'a>, HashSet<Change>>,
}

// pub static mut COUNT: usize = 0;

impl<'a> Index<'a> {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn insert(&mut self, commit: &'a CommitData) {
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

    pub fn candidates(&self) -> HashSet<CandidatePair<'a>> {
        debug!("finding util among {} entries", self.commit_index.len());
        let mut candidates = HashSet::new();
        for (i, neighbors) in self.commit_index.values().enumerate() {
            for n1 in neighbors {
                for n2 in neighbors {
                    if n1 != n2 {
                        candidates.insert(CandidatePair::new(n1, n2));
                    }
                }
            }
            if i % 1000 == 0 {
                debug!(
                    "finished search for {}/{} entries",
                    i,
                    self.commit_index.len()
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
        // TODO: uncomment
        // if c1 <= c2 {
        CandidatePair(c1, c2)
        // } else {
        //     CandidatePair(c2, c1)
        // }
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
