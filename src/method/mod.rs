use crate::git::CommitData;
use crate::CherryGroup;

pub mod diff_based;
pub mod metadata_based;

pub trait SearchMethod {
    fn search(&self, commits: &[CommitData]) -> Vec<CherryGroup>;
}

pub use metadata_based::message_scan::MessageScan;
