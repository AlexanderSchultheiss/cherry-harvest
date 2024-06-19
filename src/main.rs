#[macro_use]
extern crate log;

use cherry_harvest::git::github::ForkNetwork;
use cherry_harvest::sampling::most_stars::{MostStarsSampler, ProgrammingLanguage};
use cherry_harvest::sampling::GitHubSampler;
use cherry_harvest::{
    load_repo_sample, save_repo_sample, HarvestTracker, MessageScan, SearchMethod,
};
use log::LevelFilter;
use rayon::prelude::*;
use std::collections::HashMap;
use std::fs;
use std::path::Path;
use std::process::exit;
use std::sync::{Arc, Mutex};

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
                info!("initializing octocrab with token");
                octocrab::initialise(o);
            }
            Err(e) => {
                error!("problem while initializing octocrab: {e}");
                exit(1);
            }
        }
    }
}

// TODO: Track which repos failed to clone so that they can be retried
// TODO: Track which repository a certain commit identified as cherry or pick comes from;
// currently, we only track the seed repo of a ForkNetwork
// TODO: Update error handling to no longer panic on possible errors (address unwrap and panic)
// TODO: Update error handling so that errors are represented in the saved results
// TODO: Trace commits to all repositories and branches in which they appear in (required for analysis)
// TODO: More filter options for GitHub sampling (e.g., number of commits, number of forks)
// TODO: Try to improve performance of ANN similarity search by using FAISS
// TODO: Set up Docker
// TODO: Set up GitHub repos as fork network with known cherry-picks to validate functionality
// TODO: Plot abbreviated history with cherry-picks as graph (only show relevant events) (svg export)?
// TODO: Set up all tests to not require local repositories
// TODO: External configuration file
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

    // Top languages 2024/1 by stars
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
    // Number of repos per language
    let sample_size = 250;
    let max_forks = 0;

    info!("Starting repo sampling");
    fs::create_dir_all("output").unwrap();
    let sample_file = Path::new("output/sample.yaml");
    let sample = if Path::exists(sample_file) {
        let sample = load_repo_sample(sample_file).unwrap();
        info!("Loaded sample with {} repositories", sample.len());
        sample
    } else {
        let sample = sampler.sample(sample_size).unwrap();
        info!("Sampled {} repositories", sample.len());
        save_repo_sample(sample_file, &sample).unwrap();
        sample
    };

    let harvested_file = Path::new("output/harvested.yaml");
    let harvest_tracker = Arc::new(Mutex::new(
        HarvestTracker::load_harvest_tracker(harvested_file).unwrap(),
    ));

    let results_folder = Path::new("output/results/");
    fs::create_dir_all(results_folder).unwrap();
    let total_number_of_cherries: Arc<Mutex<HashMap<String, usize>>> =
        Arc::new(Mutex::new(HashMap::new()));
    let total_commits = Arc::new(Mutex::new(0));
    sample.into_repos().into_par_iter().for_each(|repo| {
        if harvest_tracker.lock().unwrap().contains(&repo.name) {
            // Only process repos that have not been harvested yet
            info!("already harvested {}. [skip]", repo.name);
            return;
        }
        info!("harvesting {}", repo.name);
        let message_based = Box::<MessageScan>::default() as Box<dyn SearchMethod>;
        let methods = vec![message_based];

        let repo_name = repo.name.clone();
        let repo_full_name = repo.full_name.clone();

        let network = if max_forks == 0 {
            ForkNetwork::single(repo)
        } else {
            runtime.block_on(ForkNetwork::build_from(repo, Some(max_forks)))
        };

        info!(
            "{} repositories in network of {}",
            network.len(),
            repo_full_name.as_ref().unwrap_or(&repo_name)
        );

        let (total_commits_count, results) = runtime.block_on(
            cherry_harvest::search_with_multiple(&network.repositories(), &methods),
        );

        *total_commits.lock().unwrap() += total_commits_count;

        // TODO: improve results storage
        if !results.is_empty() {
            let mut result_map = HashMap::new();
            result_map.insert("repo_name", repo_full_name.unwrap());
            result_map.insert("total_number_of_results", results.len().to_string());
            result_map.insert("total_number_of_commits", total_commits_count.to_string());
            let results = serde_yaml::to_string(&(&result_map, &results)).unwrap();
            let results_file =
                results_folder.join(Path::new(&format!("{}.yaml", &network.source().name)));
            fs::write(results_file, results).unwrap();
        }

        for result in results {
            let name = result.search_method().to_string();
            // Increment the number of results for this search method
            *total_number_of_cherries
                .lock()
                .unwrap()
                .entry(name)
                .or_default() += 1;
        }

        harvest_tracker.lock().unwrap().add(repo_name).unwrap();
    });

    let total_commits = total_commits.lock().unwrap();
    for (name, count) in total_number_of_cherries.lock().unwrap().iter() {
        info!("found a total of {count} cherry picks using {name}");
        info!("harvested from a total of {total_commits}");
    }
}
