use serde::Deserialize;
use serde::Serialize;

pub type GroundTruth = Vec<GroundTruthEntry>;

#[derive(Serialize, Deserialize, Ord, PartialOrd, Eq, PartialEq)]
pub struct GroundTruthEntry {
    pub source: CommitId,
    pub target: CommitId,
    pub method: CherryPickMethod,
    pub changed: bool,
}

#[derive(Serialize, Deserialize, Ord, PartialOrd, Eq, PartialEq)]
pub struct CommitId(pub String);

#[derive(Serialize, Deserialize, Ord, PartialOrd, Eq, PartialEq)]
pub enum CherryPickMethod {
    Manual,
    CLIGit {
        message_flagged: bool,
        conflicted: bool,
    },
    IDEGit {
        message_flagged: bool,
        conflicted: bool,
    },
}
