use crate::git::CommitData;
use crate::CherryGroup;

pub mod diff_based;
pub mod metadata_based;

pub trait Harvester {
    fn harvest(&self, commits: &Vec<CommitData>) -> Vec<CherryGroup>;
}

pub use metadata_based::message_scan::MessageScan;
