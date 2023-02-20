# cherry-search
A simple library for finding cherry-picks in git repositories. 

## Overview
With cherry-picking, it is possible to apply the changes that happened in a previous commit to the current commit. The commit that is cherry-picked is called the cherry. The cherry is usually located in another branch (source branch) and contains changes that are required in the current branch (target branch). 

![](img/cherry-pick.png)

The goal of cherry-picking - as opposed to merging - is to only apply a subset of the changes that happened on the source branch. 
In git, cherry-picking can be done with the [command line call](https://git-scm.com/docs/git-cherry-pick):
```
git cherry-pick <commit>
```

For some of our research topics it might be interesting to analyze how cherry-picking is used in practice. Currently, we have a Bachelor's thesis that is looking into cherry-picking in practice, and that is facing an unresolved challenge: git does not track cherry-picks explicitly.
This means, that there is no built-in mechanism to find all cherry-picks in a project. 

The goal of this library is to offer the necessary functionality to address this challenge. 

## Purposes
- Library that finds cherry-picks for a given set of repositories
- Repositories can be specified by a file path or URL

## Usage
### As a tool
Simply call `cargo run --release` to randomly sample GitHub fork networks for which cherry-picks are identified. 

As of v1.0.0, you can configure the search via [main.rs](src/main.rs).

### As a library

#### Harvesting specific repositories
 ```rust
 use cherry_harvest::{MessageScan, RepoLocation};
 use cherry_harvest::git::GitRepository;

fn main() {
    let method = MessageScan::default();
    // link to a test repository
    let server = "https://github.com/AlexanderSchultheiss/cherries-one".to_string();
    let results = cherry_harvest::search_with(&[&GitRepository::from(RepoLocation::Server(server))], method);
    assert_eq!(results.len(), 2);
    let expected_commits = vec![
        "b7d2e4b330165ae92e4442fb8ccfa067acd62d44",
        "018a1bde4fb5e987157a6e8f07a7d378d5f19484",
        "4e39e242712568e6f9f5b6ff113839603b722683",
        "dd594eff3dcb36e5f4bbe47176b94f6011993c71",
    ];

    for result in results {
        assert_eq!(result.search_method(), "MessageScan");
        result
            .commit_pair()
            .as_vec()
            .iter()
            .for_each(|c| assert!(expected_commits.contains(&c.id())))
    }
}
 ```

#### Harvesting random GitHub repositories
```rust
#[macro_use]
extern crate log;

use cherry_harvest::setup::sampling::{GitHubSampler, SampleRange};
use cherry_harvest::{ExactDiffMatch, MessageScan, SearchMethod, TraditionalLSH};
use chrono::NaiveDate;
use log::LevelFilter;
use std::collections::HashMap;
use std::fs;

fn init() {
    let _ = env_logger::builder()
        .is_test(true)
        .filter_level(LevelFilter::Info)
        .try_init();
}

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
    let lsh_search = Box::new(TraditionalLSH::new(8, 100, 5, 0.7)) as Box<dyn SearchMethod>;
    let methods = vec![message_based, exact_diff, lsh_search];

    for sample in sampler.take(sample_runs) {
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

            let results = serde_yaml::to_string(&results).unwrap();
            let path = format!("output/{}.yaml", network.source().name);
            fs::write(path, results).unwrap();
        }
    }
}
```