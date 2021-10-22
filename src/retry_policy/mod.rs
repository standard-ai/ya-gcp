//! A collection of traits and types used to retry asynchronous operations.
// There are several crates in the rust ecosystem that handle operation retrying; unfortunately
// none of them quite fit the needs of ya_gcp, which lead to reluctantly implementing this
// module from scratch.
//
// Many center on an API where the user submits a closure to perform the operation (e.g. Fn() ->
// Fut), and then the library runs that closure for each retry attempt. This puts fairly
// significant constraints on the nature of the operation, for example prohibiting any non-trivial
// control flow -- there's a use in the pubsub streaming subscriber where an error from the stream
// would require reconnecting the whole stream, and this is not easy to encapsulate in a closure.
//
// So ideally for ya_gcp, a retry crate would invert the control such that the caller decides
// how to perform the operation and the retry policy dictates only when/how to backoff or stop
// retrying. Some of the retry libraries are flexibile in this way (e.g. supplying
// Iterator<Item=Duration>) but have other flaws that ultimately make them a bad fit.
//
// What makes all of this more complicated is the fact that asynchronous sleeping is executor/IO
// dependent -- tokio has their timer system, async_std has another. ya_gcp is so far
// executor agnostic (although tonic in practice requires tokio, it's not a strict requirement in
// principle). The types and traits in this crate should be flexible enough for users to plug in
// arbitrary executors, although the effort to do so may be nontrivial, with useful defaults for
// at least tokio.
//
// prior art:
//
// https://docs.rs/retry/1.2.0/retry/
// pros:
// - several backoff implementations
// - flexible, can use Duration iterators instead of retry closures
// cons:
// - jitter is too aggressive (multiplies each interval by rand(0.0..1.0))
// - default parameters are too high in places, too easy of a footgun
// - does not appear actively maintained
//
// https://docs.rs/backoff/0.3.0/backoff/
// pros:
// - good exponential backoff with jittered range, max interval, etc
// - flexible, could implement Duration iterators instead of retry closures
// cons:
// - clock trait requires std::time::Instant so isn't controllable for testing
// - some default parameters are sketchy
//
// https://docs.rs/tokio-retry/0.3.0/tokio_retry/
// pros:
// - good backoff implementations, including exponential and fibonacci backoff
// cons:
// - requires tokio; we're executor agnostic
// - same overly aggressive jitter as `retry`
//
// https://docs.rs/again/0.1.2/again/
// pros:
// - exponential backoff implementation
// cons:
// - not flexible, only way to retry is submitting a closure
//
// https://docs.rs/futures-retry/0.6.0/futures_retry/
// cons:
// - not flexible, fairly rigid usage pattern
// - leaves backoff implementation to user

use std::{future::Future, time::Duration};

mod no_retry;
pub use no_retry::NoRetry;

pub mod exponential_backoff;
pub use exponential_backoff::ExponentialBackoff;

/// A policy to determine how operations should be retried after failure.
///
/// Policy implementations, along with their associated [`RetryOperation`] instances, can inform
/// callers on when to perform retries. Notably these retries are external from these types, akin
/// to [external vs internal iteration][1]. That is, the caller is responsible for actually
/// re-attempting their operation, and the policy only guides the caller on how to do so. This can
/// allow more flexible control flow in the retry path, at the expense of pushing more
/// responsibility and complexity on the user compared to internal retry mechanisms.
///
/// [1]: http://journal.stuffwithstuff.com/2013/01/13/iteration-inside-and-out
pub trait RetryPolicy<T: ?Sized, E> {
    /// The [`RetryOperation`] returned by [`new_operation`](RetryPolicy::new_operation)
    type RetryOp: RetryOperation<T, E>;

    /// Prepare to retry some operation.
    ///
    /// This can be called before or after the intial operation is executed, in preparation to
    /// begin retries. After the initial operation fails, the returned [`RetryOperation`] can be
    /// used to check whether that failure is retriable.
    ///
    /// See [`RetryOperation::check_retry`] for more details.
    fn new_operation(&mut self) -> Self::RetryOp;
}

/// A representation of retrying some specific operation instance, created by a [`RetryPolicy`].
///
/// For example, a `GET` to an endpoint might create an instance of this type after encountering an
/// error, and use this instance to repeatedly `GET` again until either the resource is returned
/// successfully, or the policy determines retries should stop.
pub trait RetryOperation<T: ?Sized, E> {
    /// A future used to delay re-attempts of the operation, to potentially reduce resource
    /// exhaustion.
    type Sleep: Future<Output = ()>;

    /// Determine whether the given operation attempt should be retried.
    ///
    /// This method will be given the contents of the attempted operation as well as the error
    /// indicating its failure. Using these two values, the retry policy will determine whether the
    /// operation should be retried. If so, the returned future will represent the period of time
    /// that should be awaited before the operation should be attempted again.
    ///
    /// If the policy determines the operation should not be retried (for example the error is
    /// terminal, or the retries have reached their configured limits), `None` will be returned.
    /// Typically the caller should then propagate the error, or otherwise handle it in some way.
    fn check_retry(&mut self, failed_value: &T, error: &E) -> Option<Self::Sleep>;
}

/// A predicate used to check whether a given error can be retried.
///
/// This is essentially a named alternative to `Fn(&E) -> bool` closures, for use in places where
/// named types are valuable.
pub trait RetryPredicate<E> {
    /// Determine whether the given error is retriable
    fn is_retriable(&self, error: &E) -> bool;
}

// blanket impl over closures
impl<F, E> RetryPredicate<E> for F
where
    F: Fn(&E) -> bool,
{
    fn is_retriable(&self, error: &E) -> bool {
        (self)(error)
    }
}

/// An implementation-provider for timed sleeping.
///
/// This is used to generically support different async runtimes' sleeping functions
pub trait Sleeper {
    /// The type of the future returned by [`sleep`](Sleeper::sleep)
    type Sleep: Future<Output = ()>;

    /// Return a future that will sleep for the given amount of time when awaited
    fn sleep(&self, time: Duration) -> Self::Sleep;
}

cfg_if::cfg_if! {
    if #[cfg(feature = "tokio")] {
        /// The default [`Sleeper`] implementation based on enabled features
        pub type DefaultSleeper = TokioSleeper;

        /// A [`Sleeper`] which uses tokio's timers to sleep
        #[derive(Debug, Clone, Default)]
        #[cfg_attr(docsrs, doc(cfg(feature = "tokio/time")))]
        pub struct TokioSleeper {
            _priv: ()
        }

        impl Sleeper for TokioSleeper {
            type Sleep = tokio::time::Sleep;

            fn sleep(&self, time: Duration) -> Self::Sleep {
                tokio::time::sleep(time)
            }
        }
    } else {
        /// The default [`Sleeper`] implementation based on enabled features
        // If tokio is not enabled, the default sleeper will not implement `Sleeper` and thus
        // structs which use it as a generic default will have to be created by some constructor
        // with a user-supplied sleeper
        pub type DefaultSleeper = NoDefaultSleeper;

        #[derive(Debug, Clone, Default)]
        #[doc(hidden)]
        pub struct NoDefaultSleeper {
            _priv: ()
        }
    }
}
