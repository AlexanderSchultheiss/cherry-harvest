use crate::git::LineType;
use crate::CommitData;
use std::collections::{HashMap, HashSet};

type Id<'a> = &'a str;
type Change = String;

#[derive(Default)]
pub struct Index<'a> {
    commit_index: HashMap<Change, HashSet<Id<'a>>>,
    change_index: HashMap<Id<'a>, HashSet<Change>>,
}

pub static mut COUNT: usize = 0;

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
                    | LineType::DelEofnl => Some(l.content().trim().to_string()),
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

    pub fn neighbors(&mut self, commit: &CommitData) -> HashSet<&'a str> {
        match self.change_index.get(commit.id()) {
            None => HashSet::new(),
            Some(changes) => {
                unsafe {
                    COUNT += changes.len();
                }
                changes
                    .iter()
                    .flat_map(|c| self.commit_index.get(c).unwrap())
                    .filter_map(|c| if *c != commit.id() { Some(*c) } else { None })
                    .collect()
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn init() {
        let _ = env_logger::builder().is_test(true).try_init();
    }

    #[test]
    fn index_search() {
        init();
    }
}
