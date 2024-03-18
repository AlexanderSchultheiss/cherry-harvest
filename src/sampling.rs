pub mod fully_random;
pub mod most_stars;
use crate::Result;

use crate::Error;
use chrono::{Duration, NaiveDate, NaiveDateTime, NaiveTime};
use fallible_iterator::FallibleIterator;
use octocrab::models::Repository;

// TODO: On-demand lazy sampling
// TODO: Retrieval of full sample
// TODO: Separate sampling of GitHub repos and ForkNetwork retrieval
// TODO: Serialization and Deserialization of samples

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

#[derive(Debug)]
pub struct Sample(Vec<Repository>);

impl Sample {
    pub fn repos(&self) -> &[Repository] {
        &self.0
    }

    pub fn into_repos(self) -> Vec<Repository> {
        self.0
    }

    pub fn len(&self) -> usize {
        self.0.len()
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
}

/// A trait for defining GitHub samplers using different sampling strategies.
pub trait GitHubSampler: FallibleIterator<Item = Repository, Error = Error> {
    /// Sample a desired number of fork networks with a desired max size.
    fn sample(&mut self, sample_size: usize) -> Result<Sample>;
}
