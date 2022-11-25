use cherry_harvest::{MessageScan, RepoLocation};
use log::LevelFilter;

const CHERRIES_ONE: &str = "https://github.com/AlexanderSchultheiss/cherries-one";

fn init() {
    let _ = env_logger::builder()
        .is_test(true)
        .filter_level(LevelFilter::Debug)
        .try_init();
}

#[test]
fn message_only() {
    init();
    //  TODO: Use ground truth
    let method = MessageScan::default();
    let groups = cherry_harvest::search_with(&RepoLocation::Server(CHERRIES_ONE), method);
    assert_eq!(groups.len(), 2);
    let expected_commits = vec![
        "b7d2e4b330165ae92e4442fb8ccfa067acd62d44",
        "018a1bde4fb5e987157a6e8f07a7d378d5f19484",
        "4e39e242712568e6f9f5b6ff113839603b722683",
        "dd594eff3dcb36e5f4bbe47176b94f6011993c71",
    ];
    for group in groups {
        assert_eq!(group.search_method, "MessageScan");
        group
            .commit_pair
            .iter()
            .for_each(|c| assert!(expected_commits.contains(&c.as_str())))
    }
}

#[test]
fn metadata_and_diff() {}

#[test]
fn diff_exact() {}

#[test]
fn diff_similarity() {}
