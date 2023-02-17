use crate::setup::github::{repo_created_in_time_range, ForkNetwork, GitHubRepo};
use crate::Error;
use chrono::{Duration, NaiveDate, NaiveDateTime, NaiveTime};
use log::{debug, info, warn};
use octocrab::models::{Repository, RepositoryId};
use rand::rngs::ThreadRng;
use rand::Rng;
use std::collections::HashSet;
use tokio::runtime::Runtime;

#[derive(Debug, Eq, PartialEq)]
pub struct SampleRange {
    start: NaiveDateTime,
    end: NaiveDateTime,
    duration: Duration,
}

impl SampleRange {
    pub fn new(start: NaiveDate, end: NaiveDate) -> Self {
        assert!(start < end);
        let duration = end - start;
        let start = NaiveDateTime::new(start, NaiveTime::default());
        let end = NaiveDateTime::new(end, NaiveTime::default());
        Self {
            start,
            end,
            duration,
        }
    }

    pub fn num_days(&self) -> i64 {
        self.duration.num_days()
    }

    pub fn num_seconds(&self) -> i64 {
        self.duration.num_seconds()
    }
}

pub struct RepoSample(Vec<ForkNetwork>);

pub struct GitHubSampler {
    previously_sampled: HashSet<RepositoryId>,
    sample_range: SampleRange,
    sample_size: usize,
    max_forks: usize,
    random: ThreadRng,
    runtime: Runtime,
}

impl GitHubSampler {
    pub fn new(sample_range: SampleRange, sample_size: usize, max_forks: usize) -> Self {
        debug!("created new GitHubSampler for the time range {sample_range:#?} and sample size {sample_size}");
        Self {
            sample_range,
            previously_sampled: HashSet::new(),
            sample_size,
            max_forks,
            random: rand::thread_rng(),
            runtime: Runtime::new().unwrap(),
        }
    }
}

impl Iterator for GitHubSampler {
    type Item = ForkNetwork;

    fn next(&mut self) -> Option<Self::Item> {
        debug!("retrieving next sample");
        let mut next = None;

        // TODO: Better way to handle this?
        let mut sample_count = 0;

        while next.is_none() {
            if sample_count > 100 {
                warn!("Found no more repositories after trying very often.");
                return None;
            }
            // To sample randomly, we add a random number of seconds to the start date
            let seconds_in_range = self.sample_range.num_seconds();
            let random_num_seconds =
                Duration::seconds(self.random.gen_range(0..(seconds_in_range + 1)));
            let random_start = self.sample_range.start + random_num_seconds;
            debug!(
                "random datetime: {}",
                random_start.format("%Y-%m-%d %H:%M:%S").to_string()
            );
            let one_hour = Duration::hours(1);
            let end = random_start + one_hour;

            let random_repo = repo_created_in_time_range(random_start, end);
            let random_repo = self.runtime.block_on(random_repo);
            match random_repo {
                Ok(Some(repo)) => {
                    debug!(
                        "found repository {} with id {} created at {}",
                        repo.name,
                        repo.id,
                        repo.created_at.unwrap()
                    );
                    match self.previously_sampled.contains(&repo.id) {
                        true => next = None,
                        false => next = Some(repo),
                    }
                }
                Err(_) => {
                    todo!()
                }
                Ok(None) => next = None,
            }
            sample_count += 1;
        }

        // Get the fork network
        match next {
            None => None,
            Some(repo) => {
                let network = ForkNetwork::build_from(repo, self.max_forks);
                // We do not want to sample the same network twice
                network.repository_ids().iter().for_each(|id| {
                    self.previously_sampled.insert(*id);
                });
                Some(network)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::setup::sampling::{GitHubSampler, SampleRange};
    use chrono::NaiveDate;
    use log::LevelFilter;

    fn init() {
        let _ = env_logger::builder()
            .is_test(true)
            .filter_level(LevelFilter::Debug)
            .try_init();
    }

    #[test]
    fn single_sample() {
        init();
        let range = SampleRange::new(
            NaiveDate::from_ymd_opt(2015, 1, 1).unwrap(),
            NaiveDate::from_ymd_opt(2022, 1, 1).unwrap(),
        );
        let mut sampler = GitHubSampler::new(range, 1, 10);
        let network = sampler.next().unwrap();
        println!("sampled {} forks", network.len())
    }
}