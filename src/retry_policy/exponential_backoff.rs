//! Items and functions to enable exponential backoff retry policies

use super::{DefaultSleeper, RetryOperation, RetryPolicy, RetryPredicate, Sleeper};
use rand::distributions::{self, Distribution};
use std::{iter::Take, time::Duration};
use tracing::debug;

/// A [`RetryPolicy`] which delays repeated retries by an increasing amount of time with each
/// attempt.
// This implementation is largely inspired by
// https://docs.rs/backoff/0.3.0/backoff/exponential/index.html
// and in turn
// https://github.com/googleapis/google-http-java-client/blob/0ddb54a0e9841309b33d8f274508ee2c8cd64412/google-http-client/src/main/java/com/google/api/client/util/ExponentialBackOff.java
// The differences are in 1) the default parameters, which here start with a smaller initial
// interval but a higher growth rate for more aggresive retries and 2) retries are capped by count
// instead of total elapsed time -- this better handles variable execution time with the operation
// itself
#[derive(Debug, Clone)]
pub struct ExponentialBackoff<R, S = DefaultSleeper> {
    retry_check: R,
    sleeper: S,
    config: Config,
    random_range: distributions::Uniform<f32>,
}

impl<R> ExponentialBackoff<R> {
    /// Create a new backoff policy which will produce retry intervals for errors that satisfy the
    /// given [`RetryPredicate`]
    ///
    /// This uses the default [`Sleeper`] implementation, which might be unavailable depending on
    /// the crate's enabled features. [`ExponentialBackoff::with_sleeper`] can be used to specify
    /// the sleeper instead of using the default
    pub fn new(retry_check: R, config: Config) -> Self {
        Self::with_sleeper(retry_check, config, DefaultSleeper::default())
    }
}

impl<R, S> ExponentialBackoff<R, S> {
    /// Create a new backoff policy which will produce retry intervals for errors that satisfy the
    /// given [`RetryPredicate`], and uses the given [`Sleeper`] to produce the interval futures.
    pub fn with_sleeper(retry_check: R, config: Config, sleeper: S) -> Self {
        Self {
            // precalculate the range because it's the same for all operations
            random_range: config.random_range(),
            retry_check,
            config,
            sleeper,
        }
    }
}

impl<T, E, R, S> RetryPolicy<T, E> for ExponentialBackoff<R, S>
where
    T: ?Sized,
    R: RetryPredicate<E> + Clone,
    S: Sleeper + Clone,
    E: std::fmt::Debug,
{
    type RetryOp = ExponentialRetry<R, S>;

    fn new_operation(&mut self) -> Self::RetryOp {
        ExponentialRetry {
            retry_check: self.retry_check.clone(),
            sleeper: self.sleeper.clone(),
            intervals: ExponentialIter::with_args(
                self.config.initial_interval,
                self.config.max_interval,
                self.config.multiplier,
                self.config.max_retries,
                self.random_range,
            ),
        }
    }
}

/// Created by [`ExponentialBackoff::new_operation`]
pub struct ExponentialRetry<R, S = DefaultSleeper> {
    retry_check: R,
    sleeper: S,
    intervals: ExponentialIter,
}

impl<T, E, R, S> RetryOperation<T, E> for ExponentialRetry<R, S>
where
    T: ?Sized,
    R: RetryPredicate<E>,
    S: Sleeper,
    E: std::fmt::Debug,
{
    type Sleep = S::Sleep;

    fn check_retry(&mut self, _val: &T, error: &E) -> Option<Self::Sleep> {
        if self.retry_check.is_retriable(error) {
            if let Some(interval) = self.intervals.next() {
                debug!(
                    message = "retrying after error",
                    ?error,
                    backoff_ms = %interval.as_millis()
                );
                return Some(self.sleeper.sleep(interval));
            } else {
                debug!(message = "exhausted retry attempts", ?error);
            }
        }
        None
    }
}

config_default! {
    /// Configuration values for [`ExponentialIter`]
    #[derive(Debug, Clone, Copy, serde::Deserialize)]
    pub struct Config {
        /// The initial delay before the first retry attempt is made
        #[serde(with = "humantime_serde")]
        @default(Duration::from_millis(10), "Config::default_initial_interval")
        pub initial_interval: Duration,

        /// The maximum delay between retry attempts.
        ///
        /// Once this interval is reached, the exponential growth will stop and this value will be
        /// returned for all subsequent polls
        #[serde(with = "humantime_serde")]
        @default(Duration::from_secs(60), "Config::default_max_interval")
        pub max_interval: Duration,

        /// The multiplication factor for the exponential growth.
        ///
        /// The delay between each retry attempt will increase by this factor for each attempted retry
        @default(2.0, "Config::default_multiplier")
        pub multiplier: f32,

        /// The number of times that retry attemps will be made before the underlying error is returned
        ///
        /// A value of `None` will cause the retries to continue indefinitely
        @default(Some(16), "Config::default_max_retries")
        pub max_retries: Option<usize>,

        /// The multiplication factor controlling the randomization applied to the retry intervals.
        ///
        /// Each retry interval will be randomly selected from a time span around the calculated
        /// exponential interval. That time span's length is determined by multiplying the
        /// calculated interval by this randomization factor, then creating a range of twice that
        /// length centered on the calculated interval. For example, with a calculated interval of 20ms
        /// and a randomization factor of 0.5, the potential range of the output interval would be
        /// `[20 * (1 - 0.5), 20 * (1 + 0.5)]` or `10ms..=30ms`.
        ///
        /// This value must be at least zero and no greater than 1. Setting the value to zero will
        /// essentially disable the randomization such that only the calculated exponential intervals
        /// are used.
        @default(0.5, "Config::default_randomization_factor")
        pub randomization_factor: f32,
    }
}

impl Config {
    fn random_range(&self) -> distributions::Uniform<f32> {
        assert!(
            (0.0..=1.0).contains(&self.randomization_factor),
            "randomization_factor must be between 0.0 and 1.0: {}",
            self.randomization_factor
        );

        distributions::Uniform::from(
            (1.0 - self.randomization_factor)..=(1.0 + self.randomization_factor),
        )
    }
}

/// An iterator which produces exponentially increasing [`std::time::Duration`] values.
///
/// See [`Config`] for settings used to control the iterator's outputs
#[derive(Debug, Clone)]
pub struct ExponentialIter {
    iter: Take<RandomRange<Exponential>>,
}

impl ExponentialIter {
    /// Create a new `ExponentialIter` with the given configuration
    pub fn new(config: Config) -> Self {
        Self::with_args(
            config.initial_interval,
            config.max_interval,
            config.multiplier,
            config.max_retries,
            config.random_range(),
        )
    }

    fn with_args(
        initial_interval: Duration,
        max_interval: Duration,
        multiplier: f32,
        max_retries: Option<usize>,
        random_range: distributions::Uniform<f32>,
    ) -> Self {
        Self {
            iter: RandomRange {
                iter: Exponential {
                    interval: initial_interval,
                    multiplier,
                    max_interval,
                },
                random_range,
            }
            .take(max_retries.unwrap_or(usize::MAX)),
        }
    }
}

impl Iterator for ExponentialIter {
    type Item = Duration;

    fn next(&mut self) -> Option<Self::Item> {
        self.iter.next()
    }
}

/// An infinite iterator whose values grow by some growth factor with every iteration, until
/// reaching a maximum which is then returned indefinitely.
#[derive(Debug, Clone)]
struct Exponential {
    interval: Duration,
    multiplier: f32,
    max_interval: Duration,
}

impl Iterator for Exponential {
    type Item = Duration;

    fn next(&mut self) -> Option<Self::Item> {
        let next = Duration::min(self.interval.mul_f32(self.multiplier), self.max_interval);
        let current = std::mem::replace(&mut self.interval, next);
        Some(current)
    }
}

/// An iterator transformer which maps each input to a value within a random range surrounding the
/// input
#[derive(Debug, Clone)]
struct RandomRange<I> {
    iter: I,
    random_range: distributions::Uniform<f32>,
}

impl<I> Iterator for RandomRange<I>
where
    I: Iterator<Item = Duration>,
{
    type Item = Duration;

    fn next(&mut self) -> Option<Self::Item> {
        Some(
            self.iter
                .next()?
                .mul_f32(self.random_range.sample(&mut rand::thread_rng())),
        )
    }
}

#[cfg(test)]
mod test {
    use super::*;

    /// With no randomization, the exponential iterator should produce exponential growth up to its
    /// max interval until its retry limit
    #[test]
    fn iter_no_random() {
        let config = Config {
            initial_interval: Duration::from_millis(10),
            max_interval: Duration::from_millis(1000),
            multiplier: 2.0,
            max_retries: Some(10),
            randomization_factor: 0.0,
        };

        let iter = ExponentialIter::new(config);

        assert_eq!(
            vec![10, 20, 40, 80, 160, 320, 640, 1000, 1000, 1000],
            iter.map(|t| Duration::as_millis(&t)).collect::<Vec<_>>()
        );
    }

    /// With randomization, the exponential iterator should produce exponential growth within
    /// certain ranges, up to its max interval until its retry limit
    #[test]
    fn iter_random() {
        let config = Config {
            initial_interval: Duration::from_millis(10),
            max_interval: Duration::from_millis(1000),
            multiplier: 2.0,
            max_retries: Some(10),
            randomization_factor: 0.1,
        };

        let iter = ExponentialIter::new(config);

        for (interval, range) in iter.map(|t| Duration::as_millis(&t)).zip(vec![
            (9..=11),
            (18..=22),
            (36..=44),
            (72..=88),
            (144..=176),
            (288..=352),
            (576..=704),
            (900..=1100),
            (900..=1100),
            (900..=1100),
        ]) {
            assert!(range.contains(&interval));
        }
    }

    #[cfg(feature = "tokio")]
    #[tokio::test]
    async fn check_retry() {
        let config = Config {
            initial_interval: Duration::from_millis(10),
            max_interval: Duration::from_millis(1000),
            multiplier: 2.0,
            max_retries: Some(3),
            randomization_factor: 0.0,
        };

        #[derive(Debug)]
        enum Retriable {
            Yes,
            No,
        }

        let mut retry_policy =
            ExponentialBackoff::new(|err: &Retriable| matches!(err, Retriable::Yes), config);

        let mut operation =
            <ExponentialBackoff<_> as RetryPolicy<(), Retriable>>::new_operation(&mut retry_policy);

        tokio::time::pause();
        let now = tokio::time::Instant::now();

        // a non-retriable error should return no retry attempt
        assert!(operation.check_retry(&(), &Retriable::No).is_none());

        // a retriable error should yield the first interval for a sleep deadline
        assert_eq!(
            operation
                .check_retry(&(), &Retriable::Yes)
                .unwrap()
                .deadline(),
            now + Duration::from_millis(10),
        );

        // a subsequent non-retriable error should return no retry attempt
        assert!(operation.check_retry(&(), &Retriable::No).is_none());

        // a following retriable error should yield the next interval
        assert_eq!(
            operation
                .check_retry(&(), &Retriable::Yes)
                .unwrap()
                .deadline(),
            now + Duration::from_millis(20),
        );

        // one more retriable error
        assert_eq!(
            operation
                .check_retry(&(), &Retriable::Yes)
                .unwrap()
                .deadline(),
            now + Duration::from_millis(40),
        );

        // now the retries have been exhausted and any error should return no interval
        assert!(operation.check_retry(&(), &Retriable::Yes).is_none());
        assert!(operation.check_retry(&(), &Retriable::No).is_none());
    }
}
