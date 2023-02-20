extern crate core;

use cherry_harvest::git::GitRepository;
use cherry_harvest::{ExactDiffMatch, SearchMethod, TraditionalLSH};
use log::{debug, info, LevelFilter};
use std::collections::HashSet;
use std::path::{Path, PathBuf};
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
fn traditional_lsh_finds_exact() {
    let start = init();
    let print = false;
    let repo = GitRepository::from(cherry_harvest::RepoLocation::Filesystem(PathBuf::from(
        Path::new("/home/alex/data/busybox/"),
    )));
    // let repo = cherry_harvest::RepoLocation::Server("https://github.com/VariantSync/DiffDetective");
    let exact_diff = Box::<ExactDiffMatch>::default() as Box<dyn SearchMethod>;
    let lsh_search = Box::new(TraditionalLSH::new(8, 100, 5, 0.7)) as Box<dyn SearchMethod>;
    let methods = vec![exact_diff, lsh_search];
    let results = cherry_harvest::search_with_multiple(&[&repo], &methods);

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
            println!("{} : {}", r.cherry().id(), r.target().id())
        }
        println!("+++++++++++++");
        println!("+++++++++++++");
        println!("+++++++++++++");
        println!("LSH:");
        for r in &lsh_results {
            println!("{} : {}", r.cherry().id(), r.target().id())
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
