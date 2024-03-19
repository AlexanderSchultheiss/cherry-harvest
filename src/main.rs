#[macro_use]
extern crate log;

use cherry_harvest::git::github::ForkNetwork;
use cherry_harvest::sampling::most_stars::{MostStarsSampler, ProgrammingLanguage};
use cherry_harvest::sampling::{GitHubSampler, SampleRange};
use cherry_harvest::{MessageScan, SearchMethod};
use log::LevelFilter;
use std::collections::HashMap;
use std::fs;
use std::process::exit;

async fn init() {
    let _ = env_logger::builder()
        .is_test(true)
        .filter_level(LevelFilter::Info)
        .try_init();

    let token = fs::read_to_string(".github-api-token").map(|s| match !s.is_empty() {
        true => Some(s.trim().to_owned()),
        false => None,
    });

    // Static initialization with a token
    if let Ok(Some(token)) = token {
        info!("found GitHub API token {}", token);
        match octocrab::Octocrab::builder().personal_token(token).build() {
            Ok(o) => {
                debug!("initializing octocrab with token");
                octocrab::initialise(o);
            }
            Err(e) => {
                error!("problem while initializing octocrab: {e}");
                exit(1);
            }
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
// TODO: Decent CLI
// TODO: Allow analysis of specific repositories
//
// Just read an interesting SCAM paper that has some nice ideas
// TODO: Check whether we can consider the hashes of blobs instead of hashes of commits. Can we
// focus on blobs overall?
// TODO: Have a look at world of code: Does it comprise information that we can use? Does it
// provide advantages over GitHub?
// TODO: WoC maps each Git repository to a central repository using the community detection
// algorithm [1]
// [1]: Mockus et al.: A complete set of related git repositories identified via community
// detection approaches based on shared commits

fn main() {
    let runtime = tokio::runtime::Runtime::new().unwrap();
    runtime.block_on(init());
    info!("starting up");
    //    let range = SampleRange::new(
    //        NaiveDate::from_ymd_opt(2010, 1, 1).unwrap(),
    //        NaiveDate::from_ymd_opt(2020, 1, 1).unwrap(),
    //    );
    let languages = vec![
        "Python".to_string(),
        "JavaScript".to_string(),
        "Go".to_string(),
        "C++".to_string(),
        "Java".to_string(),
        "TypeScript".to_string(),
        "C".to_string(),
        "C#".to_string(),
        "PHP".to_string(),
        "Rust".to_string(),
    ]
    .into_iter()
    .map(ProgrammingLanguage::new)
    .collect();

    let mut sampler = MostStarsSampler::new(languages);
    let sample_size = 10;

    let message_based = Box::<MessageScan>::default() as Box<dyn SearchMethod>;
    let methods = vec![message_based];

    info!("Starting repo sampling");
    let sample = sampler.sample(sample_size).unwrap();
    info!("Sampled {} repositories", sample.len());

    sample.into_repos().into_iter().for_each(|repo| {
        let repo_name = repo.name.clone();
        let network = ForkNetwork::build_from(repo, Some(0));
        info!(
            "{} repositories in network of {}",
            network.len(),
            &repo_name
        );

        let results = cherry_harvest::search_with_multiple(&network.repositories(), &methods);
        info!(
            "found a total of {} results for {}",
            results.len(),
            repo_name
        );

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
        if !results.is_empty() {
            let results = serde_yaml::to_string(&results).unwrap();
            let path = format!("output/{}.yaml", network.source().name);
            fs::write(path, results).unwrap();
        }
    });
}
