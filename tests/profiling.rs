use cherry_harvest::{ExactDiffMatch, RepoLocation, TraditionalLSH};
use log::{info, LevelFilter};
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
fn traditional_lsh_similarity_search() {
    let start = init();

    let call = || {
        let search_method = TraditionalLSH::new(8, 100, 5, 0.7);
        let _ = cherry_harvest::search_with(&repo_location(), search_method);
    };

    if firestorm::enabled() {
        firestorm::bench("./flames/traditional_lsh", call).unwrap();
    }

    info!("test finished in {:?}", start.elapsed())
}
