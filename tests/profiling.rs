use cherry_harvest::git::GitRepository;
use cherry_harvest::{ExactDiffMatch, RepoLocation, TraditionalLSH};
use log::{info, LevelFilter};
use std::time::Instant;

const DATASET: &str = "https://github.com/AlexanderSchultheiss/VEVOS_Simulation.git";

/// Initializes the logger and load the ground truth.
fn init() -> Instant {
    let _ = env_logger::builder()
        .is_test(true)
        .filter_level(LevelFilter::Debug)
        .try_init();
    Instant::now()
}

fn repo() -> GitRepository {
    GitRepository::from(RepoLocation::Server(DATASET.to_string()))
}

#[test]
fn message_based() {
    let start = init();

    let call = || {
        // Last search runtime was 0.0s
        let search_method = cherry_harvest::MessageScan::default();
        let _ = cherry_harvest::search_with(&[&repo()], search_method);
    };

    if firestorm::enabled() {
        firestorm::bench("./flames/message_based", call).unwrap();
    }

    info!("test finished in {:?}", start.elapsed())
}

#[test]
fn exact_match() {
    let start = init();
    let call = || {
        let search_method = ExactDiffMatch::default();
        let _ = cherry_harvest::search_with(&[&repo()], search_method);
    };

    if firestorm::enabled() {
        firestorm::bench("./flames/exact_match", call).unwrap();
    }

    info!("test finished in {:?}", start.elapsed())
}

#[test]
fn traditional_lsh_similarity_search() {
    let start = init();

    let call = || {
        let search_method = TraditionalLSH::new(8, 100, 5, 0.7);
        let _ = cherry_harvest::search_with(&[&repo()], search_method);
    };

    if firestorm::enabled() {
        firestorm::bench("./flames/traditional_lsh", call).unwrap();
    }

    info!("test finished in {:?}", start.elapsed())
}
