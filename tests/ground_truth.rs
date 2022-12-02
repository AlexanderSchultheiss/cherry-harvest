use serde::Deserialize;
use serde::Serialize;
use std::fs::File;

#[derive(Serialize, Deserialize)]
pub struct GroundTruth(Vec<GroundTruthEntry>);

impl GroundTruth {
    pub fn load(path: &str) -> Self {
        serde_yaml::from_reader(File::open(path).unwrap()).unwrap()
    }

    /// Retains only the ground truth entries that are valid for the MessageScan method
    pub fn retain_message_scan(&mut self) {
        self.0.retain(|entry| match entry.method {
            CherryPickMethod::CLIGit {
                message_flagged, ..
            }
            | CherryPickMethod::IDEGit {
                message_flagged, ..
            } => message_flagged,
            CherryPickMethod::Manual => false,
        });
    }

    /// Retains only the ground truth entries that are valid for the ExactDiffMatch method
    pub fn retain_exact_diff(&mut self) {
        self.0.retain(|entry| {
            entry.change_sets_match == SetMatch::Fully
                && entry.context_sets_match == SetMatch::Fully
        });
    }

    pub fn entries(&self) -> &Vec<GroundTruthEntry> {
        &self.0
    }
}

#[derive(Serialize, Deserialize, Ord, PartialOrd, Eq, PartialEq)]
pub struct GroundTruthEntry {
    pub source: CommitId,
    pub target: CommitId,
    pub method: CherryPickMethod,
    pub change_sets_match: SetMatch,
    pub context_sets_match: SetMatch,
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

#[derive(Serialize, Deserialize, Ord, PartialOrd, Eq, PartialEq)]
pub enum SetMatch {
    // the sets of both commits match exactly
    Fully,
    // the sets of both commits match partially (i.e., both have unique changes or context lines)
    Partially,
    // the set of the target commit is a superset of the set of the source commit
    Superset,
    // the set of the target commit is a subset of the set of the source commit
    Subset,
    // The are no commonalities
    Disjunction,
}
