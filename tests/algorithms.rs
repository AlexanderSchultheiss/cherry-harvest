mod util;

use cherry_harvest::git::GitRepository;
use cherry_harvest::{ExactDiffMatch, MessageScan, RepoLocation};
use log::{info, LevelFilter};
use util::ground_truth::GroundTruth;

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
    let runtime = tokio::runtime::Runtime::new().unwrap();
    let (_, results) = runtime
        .block_on(cherry_harvest::search_with(
            &[&GitRepository::from(RepoLocation::Server(
                CHERRIES_ONE.to_string(),
            ))],
            method,
        ))
        .unwrap();
    assert_eq!(results.len(), ground_truth.entries().len());
    let expected_commits = ground_truth
        .entries()
        .iter()
        .map(|entry| vec![entry.source.0.as_str(), entry.target.0.as_str()])
        .collect::<Vec<Vec<&str>>>();
    for result in results {
        assert_eq!(result.search_method(), "MessageScan");
        let result = result
            .commit_pair()
            .as_vec()
            .into_iter()
            .map(|c| c.id())
            .collect::<Vec<&str>>();
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
    let runtime = tokio::runtime::Runtime::new().unwrap();
    let (_, results) = runtime
        .block_on(cherry_harvest::search_with(
            &[&GitRepository::from(RepoLocation::Server(
                CHERRIES_ONE.to_string(),
            ))],
            method,
        ))
        .unwrap();
    assert_eq!(results.len(), ground_truth.entries().len());
    let expected_commits = ground_truth
        .entries()
        .iter()
        .map(|entry| vec![entry.source.0.as_str(), entry.target.0.as_str()])
        .collect::<Vec<Vec<&str>>>();
    let result_ids = results
        .iter()
        .map(|r| {
            assert_eq!(r.search_method(), "ExactDiffMatch");
            r.commit_pair()
                .as_vec()
                .into_iter()
                .map(|c| c.id())
                .collect()
        })
        .collect::<Vec<Vec<&str>>>();
    for expected in expected_commits {
        info!("checking {:#?}", expected);
        assert!(result_ids.contains(&expected));
    }
}
