use log::{debug, LevelFilter};
use std::path::Path;

/// Initializes the logger and load the ground truth.
fn init() {
    let _ = env_logger::builder()
        .is_test(true)
        .filter_level(LevelFilter::Debug)
        .try_init();
}

#[test]
fn similarity_diff_scalability() {
    init();
    let repo = cherry_harvest::RepoLocation::Filesystem(Path::new("/home/alex/data/busybox/"));
    let search_method = cherry_harvest::SimilarityDiffMatch::default();
    let _ = cherry_harvest::search_with(&repo, search_method);
}

#[test]
fn message_based_scalability() {
    init();
    let repo = cherry_harvest::RepoLocation::Filesystem(Path::new("/home/alex/data/busybox/"));
    let search_method = cherry_harvest::MessageScan::default();
    let _ = cherry_harvest::search_with(&repo, search_method);
}

#[test]
fn exact_diff_scalability() {
    init();
    let repo = cherry_harvest::RepoLocation::Filesystem(Path::new("/home/alex/data/busybox/"));
    let search_method = cherry_harvest::ExactDiffMatch::default();
    let _ = cherry_harvest::search_with(&repo, search_method);
}
