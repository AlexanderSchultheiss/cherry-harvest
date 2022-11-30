mod ground_truth;

use crate::ground_truth::{CherryPickMethod, GroundTruth};
use cherry_harvest::{MessageScan, RepoLocation};
use log::LevelFilter;
use std::fs::File;

const CHERRIES_ONE: &str = "https://github.com/AlexanderSchultheiss/cherries-one";

/// Initializes the logger and load the ground truth.
fn init() -> GroundTruth {
    let _ = env_logger::builder()
        .is_test(true)
        .filter_level(LevelFilter::Debug)
        .try_init();

    // load and return ground truth for cherries_one
    serde_yaml::from_reader(File::open("tests/resources/cherries_one_gt.yaml").unwrap()).unwrap()
}

#[test]
fn message_only() {
    let ground_truth = init();
    // filter the ground_truth for expected entries (i.e., cherry picks with message flag)
    let ground_truth = ground_truth
        .into_iter()
        .filter(|entry| match entry.method {
            CherryPickMethod::CLIGit {
                message_flagged, ..
            }
            | CherryPickMethod::IDEGit {
                message_flagged, ..
            } => message_flagged,
            CherryPickMethod::Manual => false,
        })
        .collect::<GroundTruth>();

    let method = MessageScan::default();
    let groups = cherry_harvest::search_with(&RepoLocation::Server(CHERRIES_ONE), method);
    assert_eq!(groups.len(), ground_truth.len());
    let expected_commits = ground_truth
        .iter()
        .map(|entry| vec![&entry.source.0, &entry.target.0])
        .collect::<Vec<Vec<&String>>>();
    for group in groups {
        assert_eq!(group.search_method, "MessageScan");
        let result = group.commit_pair.as_vec();
        assert!(expected_commits.contains(&result));
    }
}

#[test]
fn metadata_and_diff() {}

#[test]
fn diff_exact() {}

#[test]
fn diff_similarity() {}
