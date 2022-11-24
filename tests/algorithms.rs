use cherry_harvest;
use cherry_harvest::algorithms::MessageScan;
use std::env;

const CHERRIES_ONE: &str = "https://github.com/AlexanderSchultheiss/cherries-one";

#[test]
fn message_only() {
    let harvester = MessageScan::default();
    let cherries = cherry_harvest::search_with(CHERRIES_ONE, harvester);
    assert_eq!(cherries.len(), 2);
}

#[test]
fn metadata_and_diff() {}

#[test]
fn diff_exact() {}

#[test]
fn diff_similarity() {}
