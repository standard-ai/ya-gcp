use super::{RetryOperation, RetryPolicy};
use futures::future;

/// A retry policy where operations are never retried
#[derive(Debug, Clone, Default)]
pub struct NoRetry {
    _private: (),
}

impl NoRetry {
    /// Create a new `NoRetry` policy
    pub fn new() -> Self {
        Self { _private: () }
    }
}

impl<T, E> RetryPolicy<T, E> for NoRetry {
    type RetryOp = NoRetry;

    fn new_operation(&mut self) -> Self::RetryOp {
        NoRetry::new()
    }
}

impl<T, E> RetryOperation<T, E> for NoRetry {
    type Sleep = future::Pending<()>;

    fn check_retry(&mut self, _: &T, _: &E) -> Option<Self::Sleep> {
        None
    }
}
