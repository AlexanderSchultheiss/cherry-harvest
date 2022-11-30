mod ground_truth;

use crate::ground_truth::GroundTruth;
use cherry_harvest::{ExactDiffMatch, MessageScan, RepoLocation};
use log::LevelFilter;

const CHERRIES_ONE: &str = "https://github.com/AlexanderSchultheiss/cherries-one";

/// Initializes the logger and load the ground truth.
fn init() -> GroundTruth {
    let _ = env_logger::builder()
        .is_test(true)
        .filter_level(LevelFilter::Debug)
        .try_init();

    // load and return ground truth for cherries_one
    GroundTruth::load("tests/resources/cherries_one_gt.yaml")
}

#[test]
fn message_only() {
    let mut ground_truth = init();
    ground_truth.retain_message_scan();

    let method = MessageScan::default();
    let groups = cherry_harvest::search_with(&RepoLocation::Server(CHERRIES_ONE), method);
    assert_eq!(groups.len(), ground_truth.entries().len());
    let expected_commits = ground_truth
        .entries()
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
fn diff_exact() {
    let mut ground_truth = init();
    ground_truth.retain_exact_diff();

    let method = ExactDiffMatch::default();
    let groups = cherry_harvest::search_with(&RepoLocation::Server(CHERRIES_ONE), method);
    assert_eq!(groups.len(), ground_truth.entries().len());
    let expected_commits = ground_truth
        .entries()
        .iter()
        .map(|entry| vec![&entry.source.0, &entry.target.0])
        .collect::<Vec<Vec<&String>>>();
    for group in groups {
        assert_eq!(group.search_method, "ExactDiffMatch");
        let result = group.commit_pair.as_vec();
        assert!(expected_commits.contains(&result));
    }
}

#[test]
fn diff_similarity() {}
