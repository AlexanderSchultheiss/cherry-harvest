extern crate core;

use cherry_harvest::{ExactDiffMatch, SearchMethod, SimilarityDiffMatch};
use log::{debug, info, LevelFilter};
use std::collections::HashSet;
use std::path::Path;
use std::time::Instant;

/// Initializes the logger and load the ground truth.
fn init() -> Instant {
    let _ = env_logger::builder()
        .is_test(true)
        .filter_level(LevelFilter::Debug)
        .try_init();
    Instant::now()
}

#[test]
fn similarity_diff_scalability() {
    let start = init();
    let repo = cherry_harvest::RepoLocation::Filesystem(Path::new("/home/alex/data/busybox/"));
    let search_method = cherry_harvest::SimilarityDiffMatch::default();
    let _ = cherry_harvest::search_with(&repo, search_method);
    info!("test finished in {:?}", start.elapsed())
}

#[test]
fn message_based_scalability() {
    let start = init();
    let repo = cherry_harvest::RepoLocation::Filesystem(Path::new("/home/alex/data/busybox/"));
    let search_method = cherry_harvest::MessageScan::default();
    let _ = cherry_harvest::search_with(&repo, search_method);
    info!("test finished in {:?}", start.elapsed())
}

#[test]
fn exact_diff_scalability() {
    let start = init();
    let repo = cherry_harvest::RepoLocation::Filesystem(Path::new("/home/alex/data/busybox/"));
    let search_method = cherry_harvest::ExactDiffMatch::default();
    let _ = cherry_harvest::search_with(&repo, search_method);
    info!("test finished in {:?}", start.elapsed())
}

#[test]
fn ann_scalability() {
    let start = init();
    let repo = cherry_harvest::RepoLocation::Filesystem(Path::new("/home/alex/data/busybox/"));
    let search_method = cherry_harvest::ANNMatch::default();
    let _ = cherry_harvest::search_with(&repo, search_method);
    info!("test finished in {:?}", start.elapsed())
}

#[test]
fn brute_force_scalability() {
    let start = init();
    let repo = cherry_harvest::RepoLocation::Filesystem(Path::new("/home/alex/data/busybox/"));
    let search_method = cherry_harvest::BruteForceMatch::default();
    let _ = cherry_harvest::search_with(&repo, search_method);
    info!("test finished in {:?}", start.elapsed())
}

#[test]
fn similarity_finds_exact() {
    let start = init();
    let repo = cherry_harvest::RepoLocation::Filesystem(Path::new("/home/alex/data/busybox/"));
    let exact_diff = Box::<ExactDiffMatch>::default() as Box<dyn SearchMethod>;
    let similarity_diff = Box::<SimilarityDiffMatch>::default() as Box<dyn SearchMethod>;
    let methods = vec![exact_diff, similarity_diff];
    let results = cherry_harvest::search_with_multiple(&repo, &methods);

    let mut exact_results = HashSet::new();
    let mut sim_results = HashSet::new();
    results.iter().for_each(|r| match r.search_method() {
        "ExactDiffMatch" => {
            exact_results.insert(r.commit_pair());
        }
        "SimilarityDiffMatch" => {
            sim_results.insert(r.commit_pair());
        }
        _ => panic!("unexpected search search among results."),
    });

    sim_results.retain(|e| exact_results.contains(e));
    debug!("retained {} results", sim_results.len());

    for exact_res in exact_results {
        assert!(
            sim_results.contains(exact_res),
            "results of similarity search do not contain pair {:?}",
            exact_res
        );
    }
    info!("test finished in {:?}", start.elapsed())
}

#[test]
fn brute_force_finds_exact() {
    let start = init();
    let repo = cherry_harvest::RepoLocation::Filesystem(Path::new("/home/alex/data/busybox/"));
    let exact_diff = Box::new(cherry_harvest::ExactDiffMatch::default()) as Box<dyn SearchMethod>;
    let similarity_diff =
        Box::new(cherry_harvest::BruteForceMatch::default()) as Box<dyn SearchMethod>;
    let methods = vec![exact_diff, similarity_diff];
    let results = cherry_harvest::search_with_multiple(&repo, &methods);

    let mut exact_results = HashSet::new();
    let mut sim_results = HashSet::new();
    results.iter().for_each(|r| match r.search_method() {
        "ExactDiffMatch" => {
            exact_results.insert(r.commit_pair());
        }
        "BruteForce" => {
            sim_results.insert(r.commit_pair());
        }
        _ => panic!("unexpected search method among results."),
    });

    sim_results.retain(|e| exact_results.contains(e));
    debug!("retained {} results", sim_results.len());

    for exact_res in exact_results {
        assert!(
            sim_results.contains(exact_res),
            "results of similarity search do not contain pair {:?}",
            exact_res
        );
    }
    info!("test finished in {:?}", start.elapsed())
}
