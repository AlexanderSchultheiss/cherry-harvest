use crate::{CherryAndTarget, Commit, SearchMethod, SearchResult};
use std::collections::HashSet;

struct NaiveSearch();

const NAME: &str = "NaiveSearch";

impl SearchMethod for NaiveSearch {
    fn search(&self, commits: &mut [Commit]) -> HashSet<SearchResult> {
        let mut results: HashSet<SearchResult> = HashSet::new();
        for commit_a in commits.iter() {
            for commit_b in commits.iter() {
                // Guard against matching the same commit
                if commit_a.id() == commit_b.id() {
                    continue;
                }
                // Naively determine a cherry pick as two commits having the same commit message
                if commit_a.message() == commit_b.message() {
                    // Determine the order of the commits by their timestamp
                    let cherry_pick = CherryAndTarget::construct(commit_a, commit_b);
                    results.insert(SearchResult::new(String::from(NAME), cherry_pick));
                }
            }
        }
        results
    }

    fn name(&self) -> &'static str {
        "NAIVE_SEARCH"
    }
}
