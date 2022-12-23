//! Builders for contructing filters, used when reading bigtable rows.

use super::api::bigtable::v2;

pub use v2::row_filter::{Chain, Filter};

impl Chain {
    /// Add a new filter to the end of this chain, returning the result.
    pub fn with_filter(mut self, filter: Filter) -> Self {
        self.filters.push(v2::RowFilter {
            filter: Some(filter),
        });
        self
    }
}

impl From<Chain> for Filter {
    fn from(ch: Chain) -> Filter {
        Filter::Chain(ch)
    }
}

impl<T: Into<Filter>> From<T> for v2::RowFilter {
    fn from(f: T) -> v2::RowFilter {
        v2::RowFilter {
            filter: Some(f.into()),
        }
    }
}
