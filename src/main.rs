#[macro_use]
extern crate log;

use cherry_harvest::setup::sampling::{GitHubSampler, SampleRange};
use cherry_harvest::{ExactDiffMatch, MessageScan, SearchMethod, TraditionalLSH};
use chrono::NaiveDate;
use log::LevelFilter;
use std::collections::HashMap;

fn init() {
    let _ = env_logger::builder()
        .is_test(true)
        .filter_level(LevelFilter::Debug)
        .try_init();
}

// TODO: Update error handling to no longer panic on possible errors
// TODO: Update error handling so that errors are represented in the saved results

fn main() {
    init();
    info!("starting up");
    let range = SampleRange::new(
        NaiveDate::from_ymd_opt(2000, 1, 1).unwrap(),
        NaiveDate::from_ymd_opt(2023, 1, 1).unwrap(),
    );
    let sampler = GitHubSampler::new(range, 1, Some(20));
    let sample_runs = 1;

    let message_based = Box::<MessageScan>::default() as Box<dyn SearchMethod>;
    let exact_diff = Box::<ExactDiffMatch>::default() as Box<dyn SearchMethod>;
    let lsh_search = Box::new(TraditionalLSH::new(8, 100, 5, 0.7)) as Box<dyn SearchMethod>;
    let methods = vec![message_based, exact_diff, lsh_search];

    for sample in sampler.take(sample_runs) {
        info!("sampled {} networks", sample.networks().len());
        for (id, network) in sample.networks().iter().enumerate() {
            info!("sampled {} repositories in network {id}", network.len());
            // TODO: Integrate GitHubRepo in cherry-harvest calls
            let results = cherry_harvest::search_with_multiple(&network.repositories(), &methods);
            info!("found a total of {} results", results.len());
            let mut result_map = HashMap::new();
            results.iter().for_each(|r| {
                let method = r.search_method();
                let entry = result_map.entry(method).or_insert(vec![]);
                entry.push(r);
            });
            for (key, val) in result_map {
                info!("{key}: {}", val.len());
            }
        }
    }
}
