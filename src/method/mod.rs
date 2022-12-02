use crate::git::CommitData;
use crate::SearchResult;
use std::collections::HashSet;

pub mod diff_based;
pub mod metadata_based;

pub trait SearchMethod {
    fn search(&self, commits: &[CommitData]) -> HashSet<SearchResult>;
}

pub use diff_based::diff_exact::ExactDiffMatch;
pub use metadata_based::message_scan::MessageScan;
