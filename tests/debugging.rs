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
fn ngrammatic_scalability() {
    init();
    debug!("log test!");
    use ngrammatic::{CorpusBuilder, Pad};

    let repo = cherry_harvest::RepoLocation::Filesystem(Path::new("/home/alex/data/busybox/"));
    let search_method = cherry_harvest::SimilarityDiffMatch::default();
    let results = cherry_harvest::search_with(&repo, search_method);
}
