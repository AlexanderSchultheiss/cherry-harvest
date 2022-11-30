use crate::git::{CommitData, DiffData};
use crate::{CommitPair, SearchMethod, SearchResult};
use std::collections::HashMap;

const NAME: &str = "ExactDiffMatch";

#[derive(Default)]
pub struct ExactDiffMatch();

impl SearchMethod for ExactDiffMatch {
    fn search(&self, commits: &[CommitData]) -> Vec<SearchResult> {
        // map all commits to a hash of their diff
        let mut commit_map: HashMap<DiffData, Vec<&String>> = HashMap::new();
        commits
            .iter()
            .map(|c| (&c.diff, &c.id))
            .for_each(|(diff, id)| {
                commit_map.entry(diff.clone()).or_default().push(id);
            });

        // then, return results for all entries with more than one commit mapped to them
        // TODO: Sort by commit date to determine source and target?
        commit_map
            .iter()
            .filter_map(|(_, ids)| if ids.len() > 1 { Some(ids) } else { None })
            .flat_map(|ids| {
                let mut results = vec![];
                for (index, id) in ids.iter().enumerate() {
                    for second_id in ids[index..].iter() {
                        if id != second_id {
                            results.push(SearchResult::new(
                                NAME.to_string(),
                                CommitPair((*id).clone(), (*second_id).clone()),
                            ));
                        }
                    }
                }
                results
            })
            .collect()
    }
}
