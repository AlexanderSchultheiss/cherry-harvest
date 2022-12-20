use crate::git::LineType;
use crate::CommitData;
use std::collections::HashMap;

#[derive(Default)]
pub struct Index<'a> {
    commit_index: HashMap<String, Vec<&'a CommitData>>,
    change_index: HashMap<&'a CommitData, Vec<String>>,
}

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
                    | LineType::DelEofnl => Some(l.to_string()),
                    _ => None,
                })
            })
            .for_each(|c| {
                // update the change_index
                let entry = self.change_index.entry(commit).or_default();
                entry.push(c.clone());

                // update the commit_index
                let entry = self.commit_index.entry(c).or_default();
                entry.push(commit)
            });
    }

    pub fn neighbors(&mut self, commit: &CommitData) -> Vec<&'a CommitData> {
        let changes = self.change_index.get(commit).unwrap();
        changes
            .iter()
            .flat_map(|c| self.commit_index.get(c).unwrap())
            .filter_map(|c| {
                if c.id() != commit.id() {
                    Some(*c)
                } else {
                    None
                }
            })
            .collect()
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
