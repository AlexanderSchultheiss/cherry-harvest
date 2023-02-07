use cherry_harvest::{
    BruteForceMatch, ExactDiffMatch, RepoLocation, SearchMethod, SimilarityDiffMatch,
};
use log::{debug, info, LevelFilter};
use std::collections::HashSet;
use std::path::Path;
use std::time::Instant;

const DATASET: &'static str = "/home/alex/data/busybox/";

/// Initializes the logger and load the ground truth.
fn init() -> Instant {
    let _ = env_logger::builder()
        .is_test(true)
        .filter_level(LevelFilter::Debug)
        .try_init();
    Instant::now()
}

fn repo_location() -> RepoLocation {
    RepoLocation::Filesystem(Path::new(DATASET))
}

#[test]
#[ignore]
fn similarity_diff_scalability() {
    let start = init();
    let search_method = SimilarityDiffMatch::default();
    // Last search runtime was 116.9s (but without the custom external crate improvements that I had tried locally once)
    let _ = cherry_harvest::search_with(&repo_location(), search_method);
    // Last total runtime was 151.2s
    info!("test finished in {:?}", start.elapsed())
}

#[test]
#[ignore]
fn message_based_scalability() {
    let start = init();
    let search_method = cherry_harvest::MessageScan::default();
    // Last search runtime was 0.0s
    let _ = cherry_harvest::search_with(&repo_location(), search_method);
    // Last total runtime was 27.3s
    info!("test finished in {:?}", start.elapsed())
}

#[test]
#[ignore]
fn exact_diff_scalability() {
    let start = init();
    let search_method = ExactDiffMatch::default();
    // Last search runtime was 0.7s
    let _ = cherry_harvest::search_with(&repo_location(), search_method);
    // Last total runtime was 29s
    info!("test finished in {:?}", start.elapsed())
}

#[test]
#[ignore]
fn ann_scalability() {
    let start = init();
    let search_method = cherry_harvest::ANNMatch::default();
    // Last search runtime ... never finished ... took too long
    let _ = cherry_harvest::search_with(&repo_location(), search_method);
    // Last total runtime ... never finished ... took too long
    info!("test finished in {:?}", start.elapsed())
}

#[test]
#[ignore]
fn brute_force_scalability() {
    let start = init();
    let search_method = BruteForceMatch::default();
    // Last search runtime was ... never finished ... took too long
    let _ = cherry_harvest::search_with(&repo_location(), search_method);
    // Last total runtime was ... never finished ... took too long
    info!("test finished in {:?}", start.elapsed())
}
