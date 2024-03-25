use std::collections::HashSet;

use chrono::Duration;
use fallible_iterator::FallibleIterator;
use log::{debug, warn};
use octocrab::models::{Repository, RepositoryId};
use rand::{rngs::ThreadRng, Rng};
use tokio::runtime::Runtime;

use crate::{git::github, Result};

use super::{GitHubSampler, Sample, SampleRange};

/// This GitHub sampler selects GitHub repos by choosing a random day from the given range
/// and then choosing a random repository that was created on that day.
#[derive(Debug)]
pub struct FullyRandomSampler {
    sample_range: SampleRange,
    previously_sampled: HashSet<RepositoryId>,
    random: ThreadRng,
    runtime: Runtime,
}

impl FullyRandomSampler {
    pub fn new(sample_range: SampleRange) -> Self {
        debug!("created a new FullyRandomSampler");

        Self {
            sample_range,
            previously_sampled: HashSet::new(),
            random: rand::thread_rng(),
            runtime: Runtime::new().unwrap(),
        }
    }
}

impl GitHubSampler for FullyRandomSampler {
    fn sample(&mut self, sample_size: usize) -> Result<Sample> {
        let mut sample = Sample(Vec::with_capacity(sample_size));

        while sample.0.len() < sample_size {
            match self.next()? {
                Some(next) => sample.0.push(next),
                None => break,
            }
        }
        Ok(sample)
    }
}

impl FallibleIterator for FullyRandomSampler {
    type Item = Repository;
    type Error = crate::Error;

    fn next(&mut self) -> Result<Option<Self::Item>> {
        debug!("retrieving next repo");
        let mut next = Ok(None);

        // TODO: Better way to handle this?
        let mut sample_count = 0;

        while let Ok(None) = next {
            if sample_count > 100 {
                warn!("Found no more repositories after trying very often.");
                return Ok(None);
            }
            // To sample randomly, we add a random number of seconds to the start date
            let seconds_in_range = self.sample_range.num_seconds();
            let random_num_seconds =
                Duration::try_seconds(self.random.gen_range(0..(seconds_in_range + 1))).unwrap();
            let random_start = self.sample_range.start + random_num_seconds;
            debug!(
                "random datetime: {}",
                random_start.format("%Y-%m-%d %H:%M:%S").to_string()
            );
            let one_hour = Duration::try_hours(1).unwrap();
            let end = random_start + one_hour;

            let random_repo = self
                .runtime
                .block_on(github::repos_created_in_time_range(random_start, end));

            next = random_repo.map(|op| {
                if let Some(repo) = op {
                    if !self.previously_sampled.contains(&repo.id) {
                        debug!(
                            "found repository {} with id {} created at {}",
                            repo.name,
                            repo.id,
                            repo.created_at.unwrap()
                        );
                    }
                    Some(repo)
                } else {
                    None
                }
            });

            sample_count += 1;
        }
        next
    }
}

#[cfg(test)]
mod tests {
    use crate::sampling::{fully_random::FullyRandomSampler, GitHubSampler, SampleRange};
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
        let mut sampler = FullyRandomSampler::new(range);
        let sample = sampler.sample(2).unwrap();
        println!("sampled {} networks", sample.0.len());
        for repo in sample.0.iter() {
            println!("sampled repo {:#?}", repo.full_name);
        }
    }
}
