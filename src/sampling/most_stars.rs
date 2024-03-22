use std::collections::HashSet;
use std::rc::Rc;
use std::time::Duration;

use crate::git::github::{self, check_search_limit};
use crate::Result;
use fallible_iterator::FallibleIterator;
use log::{debug, error, info};
use octocrab::models::{Repository, RepositoryId};
use octocrab::Page;
use rand::rngs::ThreadRng;
use rand::Rng;
use tokio::runtime::Runtime;
use tokio::time;

use crate::{git::github::collect_repos_from_pages, sampling::Sample, Error};

use super::GitHubSampler;

/// The name of a programming language. Values should match the names of languages on GitHub.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProgrammingLanguage(String);

impl ProgrammingLanguage {
    pub fn new(language: String) -> ProgrammingLanguage {
        ProgrammingLanguage(language)
    }
}

/// This GitHub sampler selects the most popular repositories (indicated by stars)
/// from the given propgramming lanugages
#[derive(Debug)]
pub struct MostStarsSampler {
    languages: Vec<ProgrammingLanguage>,
    previously_sampled: HashSet<RepositoryId>,
    random: ThreadRng,
    runtime: Rc<Runtime>,
}

const THRESHOLD: f64 = 0.5;

impl MostStarsSampler {
    pub fn new(languages: Vec<ProgrammingLanguage>) -> Self {
        debug!("created a new FullyRandomSampler");

        Self {
            languages,
            random: rand::thread_rng(),
            previously_sampled: HashSet::new(),
            runtime: Rc::new(Runtime::new().unwrap()),
        }
    }

    async fn sample_for_language(
        &mut self,
        language: ProgrammingLanguage,
        sample_size: usize,
    ) -> Result<Sample> {
        info!("sampling for {}", language.0);
        let query = format!("language:{}", language.0);

        // While sample < sample_size
        let mut sample = Sample(Vec::with_capacity(sample_size));
        let mut new_repo_ratio = 1.0;
        let mut next_page = None;
        while sample.0.len() < sample_size {
            let result;
            if new_repo_ratio > THRESHOLD {
                // get repos with fresh sample request
                result = self.run_fresh_query(sample_size, &query).await.map(Some);
            } else if next_page.is_some() {
                // else if
                // get repos from next page
                result = github::get_page(&next_page).await;
            } else {
                // else
                // return current sample
                return Ok(sample);
            }
            let result = match result {
                Ok(page) => page,
                Err(error) => {
                    error!("was not able to search for repos");
                    return Err(Error::new(crate::error::ErrorKind::GitHub(error)));
                }
            };

            match result {
                Some(page) => {
                    next_page.clone_from(&page.next);
                    let repos = collect_repos_from_pages(page, Some(sample_size)).await;

                    let mut new: f64 = 0.;
                    let num_repos;
                    match repos {
                        Some(repos) => {
                            num_repos = repos.len();
                            for repo in repos {
                                if !self.previously_sampled.contains(&repo.id) {
                                    new += 1.0;
                                    self.previously_sampled.insert(repo.id);
                                    sample.0.push(repo);
                                }
                            }

                            // We collect a fresh sample, if the number of new repos is above a
                            // certain THRESHOLD. If it is below this threshold, we instead
                            // retrieve repos from the next pages in the query.
                            new_repo_ratio = new / (num_repos as f64);
                        }
                        None => return Ok(sample),
                    }
                }
                None => return Ok(sample),
            }
            info!("current sample size: {}", sample.len());
        }
        let sample = Sample(sample.0.into_iter().take(sample_size).collect());
        info!("sampled {} repos for {}", sample.len(), language.0);
        let seconds = 60;
        info!("sleeping for {seconds} seconds to reduce API stress...");
        time::sleep(Duration::from_secs(seconds)).await;
        info!("continuing");
        Ok(sample)
    }

    async fn run_fresh_query(
        &self,
        sample_size: usize,
        query: &str,
    ) -> std::result::Result<Page<Repository>, octocrab::Error> {
        check_search_limit().await.unwrap();
        // GitHub allows up to 100 results per page
        let results_per_page = usize::max(sample_size, 100) as u8 /*safe cast*/;
        let sort = "stars";
        let order = "desc";
        info!("run_fresh_query");
        check_search_limit().await.unwrap();
        octocrab::instance()
            .search()
            .repositories(query)
            .sort(sort)
            .order(order)
            .per_page(results_per_page)
            .page(0u32)
            .send()
            .await
    }
}

impl GitHubSampler for MostStarsSampler {
    fn sample(&mut self, sample_size: usize) -> Result<Sample> {
        let runtime = Rc::clone(&self.runtime);
        let mut sample = Sample(Vec::with_capacity(sample_size * self.languages.len()));
        for language in self.languages.clone() {
            let s = runtime.block_on(self.sample_for_language(language, sample_size))?;
            sample.0.extend(s.0.into_iter());
        }

        // Clear, because a new sample call should start with the initial state
        self.previously_sampled.clear();
        Ok(sample)
    }
}

impl FallibleIterator for MostStarsSampler {
    type Item = Repository;

    type Error = crate::Error;

    fn next(&mut self) -> core::result::Result<Option<Self::Item>, Self::Error> {
        let runtime = Rc::clone(&self.runtime);
        let language_number = self.random.gen_range(0..self.languages.len());
        let language = self.languages[language_number].clone();

        // Sample one entry for a randomly selected language
        let sample = runtime.block_on(self.sample_for_language(language, 1));
        sample.map(|mut s| s.0.pop())
    }
}
