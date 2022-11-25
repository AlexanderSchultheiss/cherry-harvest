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
    let harvester = MessageScan::default();
    let groups = cherry_harvest::search_with(&RepoLocation::Server(CHERRIES_ONE), harvester);
    assert_eq!(groups.len(), 2);
    for group in groups {
        assert_eq!(group.search_method, "MessageScan")
    }
}

#[test]
fn metadata_and_diff() {}

#[test]
fn diff_exact() {}

#[test]
fn diff_similarity() {}
