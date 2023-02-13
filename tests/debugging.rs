extern crate core;

use cherry_harvest::{ANNMatch, ExactDiffMatch, HNSWSearch, SearchMethod, SimilarityDiffMatch, TraditionalLSH};
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
#[ignore]
fn hora_finds_exact() {
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
        _ => panic!("unexpected search method among results."),
    });

    sim_results.retain(|e| exact_results.contains(e));
    debug!("retained {} results", sim_results.len());

    for exact_res in exact_results {
        assert!(
            sim_results.contains(exact_res),
            "results of similarity search do not contain pair {exact_res:?}"
        );
    }
    info!("test finished in {:?}", start.elapsed())
}

#[test]
#[ignore]
fn ann_finds_exact() {
    let start = init();
    let repo = cherry_harvest::RepoLocation::Filesystem(Path::new("/home/alex/data/busybox/"));
    let exact_diff = Box::<ExactDiffMatch>::default() as Box<dyn SearchMethod>;
    let ann_search = Box::<ANNMatch>::default() as Box<dyn SearchMethod>;
    let methods = vec![exact_diff, ann_search];
    let results = cherry_harvest::search_with_multiple(&repo, &methods);

    let mut exact_results = HashSet::new();
    let mut sim_results = HashSet::new();
    results.iter().for_each(|r| match r.search_method() {
        "ExactDiffMatch" => {
            exact_results.insert(r.commit_pair());
        }
        "ANN" => {
            sim_results.insert(r.commit_pair());
        }
        _ => panic!("unexpected search method among results."),
    });

    sim_results.retain(|e| exact_results.contains(e));
    debug!("retained {} results", sim_results.len());

    for exact_res in exact_results {
        assert!(
            sim_results.contains(exact_res),
            "results of similarity search do not contain pair {exact_res:?}"
        );
    }
    info!("test finished in {:?}", start.elapsed())
}

#[test]
#[ignore]
fn hnsw_finds_exact() {
    let start = init();
    let repo = cherry_harvest::RepoLocation::Filesystem(Path::new("/home/alex/data/busybox/"));
    let exact_diff = Box::<ExactDiffMatch>::default() as Box<dyn SearchMethod>;
    let hnsw_search = Box::<HNSWSearch>::default() as Box<dyn SearchMethod>;
    let methods = vec![exact_diff, hnsw_search];
    let results = cherry_harvest::search_with_multiple(&repo, &methods);

    let mut exact_results = HashSet::new();
    let mut hnsw_results = HashSet::new();
    results.iter().for_each(|r| match r.search_method() {
        "ExactDiffMatch" => {
            exact_results.insert(r.commit_pair());
        }
        "HNSW" => {
            hnsw_results.insert(r.commit_pair());
        }
        _ => panic!("unexpected search method among results."),
    });

    hnsw_results.retain(|e| exact_results.contains(e));
    debug!("retained {} results", hnsw_results.len());

    for exact_res in exact_results {
        assert!(
            hnsw_results.contains(exact_res),
            "results of similarity search do not contain pair {exact_res:?}"
        );
    }
    info!("test finished in {:?}", start.elapsed())
}

#[test]
#[ignore]
fn traditional_lsh_finds_exact() {
    let start = init();
    let print = true;
    let repo = cherry_harvest::RepoLocation::Filesystem(Path::new("/home/alex/data/busybox/"));
    // let repo = cherry_harvest::RepoLocation::Server("https://github.com/VariantSync/DiffDetective");
    let exact_diff = Box::<ExactDiffMatch>::default() as Box<dyn SearchMethod>;
    let lsh_search = Box::new(TraditionalLSH::new(8, 100, 24, 5, 0.7)) as Box<dyn SearchMethod>;
    let methods = vec![exact_diff, lsh_search];
    let results = cherry_harvest::search_with_multiple(&repo, &methods);

    let mut exact_results = HashSet::new();
    let mut lsh_results = HashSet::new();
    results.iter().for_each(|r| match r.search_method() {
        "ExactDiffMatch" => {
            exact_results.insert(r.commit_pair());
        }
        "TraditionalLSH" => {
            lsh_results.insert(r.commit_pair());
        }
        _ => panic!("unexpected search method among results."),
    });

    if print {
        println!("EXACT:");
        for r in &exact_results {
            println!("{} : {}", r.cherry(), r.target())
        }
        println!("+++++++++++++");
        println!("+++++++++++++");
        println!("+++++++++++++");
        println!("LSH:");
        for r in &lsh_results {
            println!("{} : {}", r.cherry(), r.target())
        }
    }

    lsh_results.retain(|e| exact_results.contains(e));
    debug!("retained {} results", lsh_results.len());

    for exact_res in exact_results {
        assert!(
            lsh_results.contains(exact_res),
            "results of similarity search do not contain pair {exact_res:?}"
        );
    }
    info!("test finished in {:?}", start.elapsed())
}
