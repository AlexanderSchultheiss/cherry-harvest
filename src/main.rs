#[macro_use]
extern crate log;

use cherry_harvest::setup::sampling::{GitHubSampler, SampleRange};
use cherry_harvest::{ExactDiffMatch, SearchMethod, TraditionalLSH};
use chrono::NaiveDate;
use log::LevelFilter;

fn init() {
    let _ = env_logger::builder()
        .is_test(true)
        .filter_level(LevelFilter::Debug)
        .try_init();
}

fn main() {
    init();
    info!("starting up");
    let range = SampleRange::new(
        NaiveDate::from_ymd_opt(2000, 1, 1).unwrap(),
        NaiveDate::from_ymd_opt(2023, 1, 1).unwrap(),
    );
    let mut sampler = GitHubSampler::new(range, 10, Some(50));
    let sample_runs = 10;

    let exact_diff = Box::<ExactDiffMatch>::default() as Box<dyn SearchMethod>;
    let lsh_search = Box::new(TraditionalLSH::new(8, 100, 5, 0.7)) as Box<dyn SearchMethod>;
    let methods = vec![exact_diff, lsh_search];

    for sample in sampler.take(sample_runs) {
        info!("sampled {} networks", sample.networks().len());
        for (id, network) in sample.networks().iter().enumerate() {
            info!("sampled {} repositories in network {id}", network.len());
            // TODO: Integrate GitHubRepo in cherry-harvest calls
            // let results = cherry_harvest::search_with_multiple(&network.repositories(), &methods);
        }
    }
}
