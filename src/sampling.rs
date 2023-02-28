use crate::git::github::{repos_created_in_time_range, ForkNetwork};
use chrono::{Duration, NaiveDate, NaiveDateTime, NaiveTime};
use log::{debug, warn};
use octocrab::models::RepositoryId;
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

pub struct Sample(Vec<ForkNetwork>);

impl Sample {
    pub fn networks(&self) -> &Vec<ForkNetwork> {
        &self.0
    }
}

pub struct GitHubSampler {
    previously_sampled: HashSet<RepositoryId>,
    sample_range: SampleRange,
    sample_size: usize,
    max_forks: Option<usize>,
    random: ThreadRng,
    runtime: Runtime,
}

impl GitHubSampler {
    pub fn new(sample_range: SampleRange, sample_size: usize, max_forks: Option<usize>) -> Self {
        debug!("created new GitHubSampler for the time range {sample_range:#?} and sample size {sample_size}");
        let sampler = Self {
            sample_range,
            previously_sampled: HashSet::new(),
            sample_size,
            max_forks,
            random: rand::thread_rng(),
            runtime: Runtime::new().unwrap(),
        };
        sampler
    }
}

impl Iterator for GitHubSampler {
    type Item = Sample;

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

            let random_repo = self.runtime.block_on(repos_created_in_time_range(
                self.sample_size,
                random_start,
                end,
            ));

            match random_repo {
                Ok(Some(mut repos)) => {
                    repos.retain(|repo| !self.previously_sampled.contains(&repo.id));
                    for repo in &repos {
                        debug!(
                            "found repository {} with id {} created at {}",
                            repo.name,
                            repo.id,
                            repo.created_at.unwrap()
                        );
                    }
                    if !repos.is_empty() {
                        next = Some(repos);
                    }
                }
                Err(_) => {
                    todo!()
                }
                Ok(None) => next = None,
            }
            sample_count += 1;
        }

        // Get the fork networks
        match next {
            None => None,
            Some(repos) => {
                let mut networks = vec![];
                for repo in repos {
                    let network = ForkNetwork::build_from(repo, self.max_forks);
                    // We do not want to sample the same network twice
                    network.repository_ids().iter().for_each(|id| {
                        self.previously_sampled.insert(*id);
                    });
                    networks.push(network)
                }

                Some(Sample(networks))
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::sampling::{GitHubSampler, SampleRange};
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
            NaiveDate::from_ymd_opt(2022, 1, 1).unwrap(),
            NaiveDate::from_ymd_opt(2022, 1, 2).unwrap(),
        );
        let mut sampler = GitHubSampler::new(range, 2, None);
        let sample = sampler.next().unwrap();
        println!("sampled {} networks", sample.0.len());
        for (id, network) in sample.0.iter().enumerate() {
            println!("sampled {} repositories in network {id}", network.len());
            println!("{}", network);
        }
    }
}
