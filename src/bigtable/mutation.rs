//! Types and builders for requesting bigtable mutations.

pub use super::api::bigtable::v2::{
    self, mutate_rows_request::Entry, mutation::*, MutateRowRequest, MutateRowsRequest,
};

use super::Bytes;

impl SetCell {
    /// Create a new `SetCell` mutation for setting the value of a single cell.
    pub fn new(family_name: String, column_qualifier: Bytes, value: Bytes) -> SetCell {
        SetCell {
            family_name,
            column_qualifier,
            value,
            ..Default::default()
        }
    }

    /// Set the timestamp value of this `SetCell` mutation.
    ///
    /// The special value `-1` means that the
    /// [server provides the timestamp](https://cloud.google.com/bigtable/docs/reference/data/rpc/google.bigtable.v2#google.bigtable.v2.Mutation.SetCell).
    pub fn with_timestamp(mut self, timestamp_micros: i64) -> Self {
        self.timestamp_micros = timestamp_micros;
        self
    }
}

impl Entry {
    /// Create a new `Entry` for modifying the given row.
    pub fn new(row_key: Bytes) -> Self {
        Entry {
            row_key,
            ..Default::default()
        }
    }

    /// Add a mutation to this Entry.
    pub fn with_mutation(mut self, mutation: impl Into<Mutation>) -> Self {
        self.mutations.push(super::v2::Mutation {
            mutation: Some(mutation.into()),
        });
        self
    }

    /// Add a list of mutations to this Entry.
    pub fn with_mutations(
        mut self,
        mutations: impl IntoIterator<Item = impl Into<Mutation>>,
    ) -> Self {
        self.mutations
            .extend(mutations.into_iter().map(|mutation| super::v2::Mutation {
                mutation: Some(mutation.into()),
            }));
        self
    }
}

impl From<SetCell> for Mutation {
    fn from(sc: SetCell) -> Mutation {
        Mutation::SetCell(sc)
    }
}

impl From<DeleteFromColumn> for Mutation {
    fn from(dfc: DeleteFromColumn) -> Mutation {
        Mutation::DeleteFromColumn(dfc)
    }
}

impl From<DeleteFromFamily> for Mutation {
    fn from(dff: DeleteFromFamily) -> Mutation {
        Mutation::DeleteFromFamily(dff)
    }
}

impl From<DeleteFromRow> for Mutation {
    fn from(dfr: DeleteFromRow) -> Mutation {
        Mutation::DeleteFromRow(dfr)
    }
}

/// The error returned from a row mutation request.
#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
pub enum MutateRowsError {
    /// A gRPC error.
    #[error("gRPC error: {0}")]
    Tonic(#[from] tonic::Status),

    /// An error mutating one or more specific rows.
    #[error("row mutation error")]
    RowErrors(Vec<v2::mutate_rows_response::Entry>),
}

impl From<Vec<v2::mutate_rows_response::Entry>> for MutateRowsError {
    fn from(entries: Vec<v2::mutate_rows_response::Entry>) -> Self {
        MutateRowsError::RowErrors(entries)
    }
}

impl MutateRowRequest {
    /// Create a new request for modifying a specific row.
    pub fn new(table_name: String, row_key: Bytes) -> MutateRowRequest {
        MutateRowRequest {
            table_name,
            row_key,
            ..Default::default()
        }
    }

    /// Add a mutation to this request.
    pub fn with_mutation(mut self, mutation: impl Into<Mutation>) -> Self {
        self.mutations.push(v2::Mutation {
            mutation: Some(mutation.into()),
        });
        self
    }

    /// Add a list of mutations to this request.
    pub fn with_mutations(
        mut self,
        mutations: impl IntoIterator<Item = impl Into<Mutation>>,
    ) -> Self {
        self.mutations
            .extend(mutations.into_iter().map(|mutation| v2::Mutation {
                mutation: Some(mutation.into()),
            }));
        self
    }
}

impl MutateRowsRequest {
    /// Create a new request for modifying multiple rows.
    ///
    /// See [Google's docs](https://cloud.google.com/bigtable/docs/writing-data#batch) for advice
    /// about when to batch writes.
    pub fn new(table_name: String) -> MutateRowsRequest {
        MutateRowsRequest {
            table_name,
            ..Default::default()
        }
    }

    /// Add an entry to this request.
    pub fn with_entry(mut self, entry: Entry) -> Self {
        self.entries.push(entry);
        self
    }

    /// Add a list of entries to this request.
    pub fn with_entries(mut self, entries: impl Iterator<Item = Entry>) -> Self {
        self.entries.extend(entries);
        self
    }
}
