use crate::setup::github::{first_repo_pushed_after_datetime, ForkNetwork, GitHubRepo};
use crate::Error;
use chrono::{Duration, NaiveDate, NaiveDateTime, NaiveTime};
use log::warn;
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
    previously_sampled: HashSet<String>,
    sample_range: SampleRange,
    sample_size: usize,
    random: ThreadRng,
    runtime: Runtime,
}

impl GitHubSampler {
    pub fn new(sample_range: SampleRange, sample_size: usize) -> Self {
        Self {
            sample_range,
            previously_sampled: HashSet::new(),
            sample_size,
            random: rand::thread_rng(),
            runtime: Runtime::new().unwrap(),
        }
    }
}

impl Iterator for GitHubSampler {
    type Item = GitHubRepo;

    fn next(&mut self) -> Option<Self::Item> {
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
            let random_datetime = self.sample_range.start + random_num_seconds;
            let random_repo = first_repo_pushed_after_datetime(random_datetime);
            let random_repo = self.runtime.block_on(random_repo);
            match random_repo {
                Ok(repo) => next = repo,
                Err(_) => {
                    todo!()
                }
            }
            sample_count += 1;
        }
        next
    }
}
