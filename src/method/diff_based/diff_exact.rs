use crate::git::CommitData;
use crate::{SearchMethod, SearchResult};

const NAME: &str = "ExactDiffMatch";

#[derive(Default)]
pub struct ExactDiffMatch();

impl SearchMethod for ExactDiffMatch {
    fn search(&self, commits: &[CommitData]) -> Vec<SearchResult> {
        todo!()
    }
}
