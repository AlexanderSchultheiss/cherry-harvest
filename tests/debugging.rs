extern crate core;

use cherry_harvest::SearchMethod;
use log::{debug, LevelFilter};
use std::collections::HashSet;
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

#[test]
fn ann_scalability() {
    init();
    let repo = cherry_harvest::RepoLocation::Filesystem(Path::new("/home/alex/data/busybox/"));
    let search_method = cherry_harvest::ANNMatch::default();
    let _ = cherry_harvest::search_with(&repo, search_method);
}

#[test]
fn brute_force_scalability() {
    init();
    let repo = cherry_harvest::RepoLocation::Filesystem(Path::new("/home/alex/data/busybox/"));
    let search_method = cherry_harvest::BruteForceMatch::default();
    let _ = cherry_harvest::search_with(&repo, search_method);
}

#[test]
fn similarity_finds_exact() {
    init();
    let repo = cherry_harvest::RepoLocation::Filesystem(Path::new("/home/alex/data/busybox/"));
    let exact_diff = Box::new(cherry_harvest::ExactDiffMatch::default()) as Box<dyn SearchMethod>;
    let similarity_diff =
        Box::new(cherry_harvest::SimilarityDiffMatch::default()) as Box<dyn SearchMethod>;
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
}

#[test]
fn brute_force_finds_exact() {
    init();
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
}
