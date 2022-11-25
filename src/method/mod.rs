use crate::git::CommitData;
use crate::CherryGroup;

pub mod diff_based;
pub mod metadata_based;

pub trait SearchMethod {
    fn search(&self, commits: &Vec<CommitData>) -> Vec<CherryGroup>;
}

pub use metadata_based::message_scan::MessageScan;

// #[derive(Eq, PartialEq, Debug)]
// pub enum SearchMethod {
//     MessageScan,
//     ExactDiff,
//     Similarity,
//     Metadata,
// }
