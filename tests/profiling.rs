use cherry_harvest::{BruteForceMatch, ExactDiffMatch, RepoLocation, SimilarityDiffMatch};
use log::{debug, info, LevelFilter};
use std::path::Path;
use std::time::Instant;

const DATASET: &str = "/home/alex/data/VEVOS_Simulation/";

/// Initializes the logger and load the ground truth.
fn init() -> Instant {
    let _ = env_logger::builder()
        .is_test(true)
        .filter_level(LevelFilter::Debug)
        .try_init();
    Instant::now()
}

fn repo_location() -> RepoLocation<'static> {
    RepoLocation::Filesystem(Path::new(DATASET))
}

#[test]
#[ignore]
fn message_based() {
    let start = init();

    let call = || {
        // Last search runtime was 0.0s
        let search_method = cherry_harvest::MessageScan::default();
        let _ = cherry_harvest::search_with(&repo_location(), search_method);
    };

    if firestorm::enabled() {
        firestorm::bench("./flames/message_based", call).unwrap();
    }

    info!("test finished in {:?}", start.elapsed())
}

#[test]
#[ignore]
fn exact_match() {
    let start = init();
    let call = || {
        let search_method = ExactDiffMatch::default();
        let _ = cherry_harvest::search_with(&repo_location(), search_method);
    };

    if firestorm::enabled() {
        firestorm::bench("./flames/exact_match", call).unwrap();
    }

    info!("test finished in {:?}", start.elapsed())
}

#[test]
#[ignore]
fn hora_similarity_search() {
    let start = init();

    let call = || {
        let search_method = SimilarityDiffMatch::default();
        let results = cherry_harvest::search_with(&repo_location(), search_method);
    };

    if firestorm::enabled() {
        firestorm::bench("./flames/hora_similarity", call).unwrap();
    }

    info!("test finished in {:?}", start.elapsed())
}

#[test]
#[ignore]
fn ann_similarity_search() {
    let start = init();

    let call = || {
        let search_method = cherry_harvest::ANNMatch::default();
        let _ = cherry_harvest::search_with(&repo_location(), search_method);
    };

    if firestorm::enabled() {
        firestorm::bench("./flames/ann_similarity", call).unwrap();
    }

    info!("test finished in {:?}", start.elapsed())
}

#[test]
#[ignore]
fn hnsw_similarity_search() {
    let start = init();

    let call = || {
        let search_method = cherry_harvest::HNSWSearch::default();
        let _ = cherry_harvest::search_with(&repo_location(), search_method);
    };

    if firestorm::enabled() {
        firestorm::bench("./flames/hnsw_similarity", call).unwrap();
    }

    info!("test finished in {:?}", start.elapsed())
}

#[test]
#[ignore]
fn brute_force_similarity_search() {
    let start = init();

    let call = || {
        let search_method = BruteForceMatch::default();
        let _ = cherry_harvest::search_with(&repo_location(), search_method);
    };

    if firestorm::enabled() {
        firestorm::bench("./flames/brute_force_similarity", call).unwrap();
    }

    info!("test finished in {:?}", start.elapsed())
}

#[test]
#[ignore]
fn traditional_lsh_similarity_search() {
    let start = init();

    let call = || {
        let search_method = cherry_harvest::TraditionalLSH::new(3, 2048, 24, 2, 0.7);
        let _ = cherry_harvest::search_with(&repo_location(), search_method);
    };

    if firestorm::enabled() {
        firestorm::bench("./flames/traditional_lsh", call).unwrap();
    }

    info!("test finished in {:?}", start.elapsed())
}
