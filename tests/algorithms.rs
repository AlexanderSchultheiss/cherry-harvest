mod ground_truth;

use crate::ground_truth::GroundTruth;
use cherry_harvest::{ANNMatch, ExactDiffMatch, MessageScan, RepoLocation, SimilarityDiffMatch};
use log::{info, LevelFilter};

const CHERRIES_ONE: &str = "https://github.com/AlexanderSchultheiss/cherries-one.git";

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
    let results = cherry_harvest::search_with(&RepoLocation::Server(CHERRIES_ONE), method);
    assert_eq!(results.len(), ground_truth.entries().len());
    let expected_commits = ground_truth
        .entries()
        .iter()
        .map(|entry| vec![&entry.source.0, &entry.target.0])
        .collect::<Vec<Vec<&String>>>();
    for result in results {
        assert_eq!(result.search_method(), "MessageScan");
        let result = result.commit_pair().as_vec();
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
    let results = cherry_harvest::search_with(&RepoLocation::Server(CHERRIES_ONE), method);
    assert_eq!(results.len(), ground_truth.entries().len());
    let expected_commits = ground_truth
        .entries()
        .iter()
        .map(|entry| vec![&entry.source.0, &entry.target.0])
        .collect::<Vec<Vec<&String>>>();
    let result_ids = results
        .iter()
        .map(|r| {
            assert_eq!(r.search_method(), "ExactDiffMatch");
            r.commit_pair().as_vec()
        })
        .collect::<Vec<Vec<&String>>>();
    for expected in expected_commits {
        info!("checking {:#?}", expected);
        assert!(result_ids.contains(&expected));
    }
}

#[test]
fn diff_similarity() {
    let ground_truth = init();

    let method = SimilarityDiffMatch::default();
    let results = cherry_harvest::search_with(&RepoLocation::Server(CHERRIES_ONE), method);
    assert_eq!(results.len(), ground_truth.entries().len());
    let expected_commits = ground_truth
        .entries()
        .iter()
        .map(|entry| vec![&entry.source.0, &entry.target.0])
        .collect::<Vec<Vec<&String>>>();
    let result_ids = results
        .iter()
        .map(|r| {
            assert_eq!(r.search_method(), "SimilarityDiffMatch");
            r.commit_pair().as_vec()
        })
        .collect::<Vec<Vec<&String>>>();
    for expected in expected_commits {
        info!("checking {:#?}", expected);
        assert!(result_ids.contains(&expected));
    }
}

#[test]
fn diff_ann() {
    let ground_truth = init();

    let method = ANNMatch::default();
    let results = cherry_harvest::search_with(&RepoLocation::Server(CHERRIES_ONE), method);
    assert_eq!(results.len(), ground_truth.entries().len());
    let expected_commits = ground_truth
        .entries()
        .iter()
        .map(|entry| vec![&entry.source.0, &entry.target.0])
        .collect::<Vec<Vec<&String>>>();
    let result_ids = results
        .iter()
        .map(|r| {
            assert_eq!(r.search_method(), "SimilarityDiffMatch");
            r.commit_pair().as_vec()
        })
        .collect::<Vec<Vec<&String>>>();
    for expected in expected_commits {
        info!("checking {:#?}", expected);
        assert!(result_ids.contains(&expected));
    }
}
