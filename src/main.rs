#[macro_use]
extern crate log;
use cherry_harvest::sampling::{GitHubSampler, SampleRange};
#[cfg(feature = "faiss")]
use cherry_harvest::search::methods::random_projections_lsh::RandomProjectionsLSH;
use cherry_harvest::{ExactDiffMatch, MessageScan, SearchMethod, TraditionalLSH};
use chrono::NaiveDate;
use log::LevelFilter;
use std::collections::HashMap;
use std::fs;
use std::process::exit;

fn init() {
    let _ = env_logger::builder()
        .is_test(true)
        .filter_level(LevelFilter::Debug)
        .try_init();

    let token = fs::read_to_string(".authentication").map(|s| match !s.is_empty() {
        true => Some(s),
        false => None,
    });

    // Static initialization with a token
    if let Ok(Some(token)) = token {
        if let Err(error) =
            octocrab::initialise(octocrab::Octocrab::builder().personal_token(token))
        {
            error!("problem while initializing octocrab: {error}");
            exit(1);
        }
    }
}

// TODO: Update error handling to no longer panic on possible errors (address unwrap and panic)
// TODO: Update error handling so that errors are represented in the saved results
// TODO: Trace commits to all repositories and branches in which they appear in (required for analysis)
// TODO: More filter options for GitHub sampling (e.g., number of commits, number of forks)
// TODO: Create GitHub user for cherry-harvest and sign into octocrab for more requests/minute
// TODO: Control request rate to GitHub to prevent limit reached errors
// TODO: Try to improve performance of ANN similarity search by using FAISS
// TODO: Set up Docker
// TODO: Set up GitHub repos as fork network with known cherry-picks to validate functionality
// TODO: Plot abbreviated history with cherry-picks as graph (only show relevant events) (svg export)?
// TODO: Set up all tests to not require local repositories
// TODO: External configuration file
// TODO: Reduce type overhead: the lib is working with three different commit types and three different repository types

fn main() {
    init();
    info!("starting up");
    let range = SampleRange::new(
        NaiveDate::from_ymd_opt(2000, 1, 1).unwrap(),
        NaiveDate::from_ymd_opt(2023, 1, 1).unwrap(),
    );
    let sampler = GitHubSampler::new(range, 1, Some(5));
    let sample_runs = 1;

    let message_based = Box::<MessageScan>::default() as Box<dyn SearchMethod>;
    let exact_diff = Box::<ExactDiffMatch>::default() as Box<dyn SearchMethod>;
    let lsh_traditional = Box::new(TraditionalLSH::new(8, 100, 5, 0.7)) as Box<dyn SearchMethod>;
    let methods = vec![message_based, exact_diff, lsh_traditional];

    #[cfg(feature = "faiss")]
    let lsh_random = Box::new(RandomProjectionsLSH::new(35, 24, 0.7)) as Box<dyn SearchMethod>;
    #[cfg(feature = "faiss")]
    let mut methods = methods;
    #[cfg(feature = "faiss")]
    methods.push(lsh_random);

    sampler.take(sample_runs).for_each(|sample| {
        info!("sampled {} networks", sample.networks().len());
        for (id, network) in sample.networks().iter().enumerate() {
            info!("sampled {} repositories in network {id}", network.len());

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

            // TODO: improve results storage
            let results = serde_yaml::to_string(&results).unwrap();
            let path = format!("output/{}.yaml", network.source().name);
            fs::write(path, results).unwrap();
        }
    });
}
