//! An API for interacting with Google's [Bigtable](https://cloud.google.com/bigtable) database.
//!
//! The [`BigtableClient`] allows reading and writing to tables, while the
//! [`BigtableTableAdminClient`](admin::BigtableTableAdminClient)
//! allows for creating and listing tables.

use futures::prelude::*;
use hyper::body::Bytes;
use prost::bytes::BytesMut;
use std::ops::{Bound, RangeBounds};

use crate::auth::grpc::AuthGrpcService;
use crate::retry_policy::{
    exponential_backoff, ExponentialBackoff, RetryOperation, RetryPolicy, RetryPredicate,
};

pub use http::Uri;
pub use tower::make::MakeConnection;

pub mod admin;
mod client_builder;
pub mod filters;
pub mod mutation;

pub use client_builder::{BigtableConfig, BuildError};
pub use mutation::{MutateRowRequest, MutateRowsError, MutateRowsRequest};

#[cfg(feature = "emulators")]
#[cfg_attr(docsrs, doc(cfg(feature = "emulators")))]
pub mod emulator;

#[allow(rustdoc::broken_intra_doc_links, rustdoc::bare_urls, missing_docs)]
pub mod api {
    pub mod rpc {
        include!("../generated/google.rpc.rs");
    }
    pub mod longrunning {
        include!("../generated/google.longrunning.rs");
    }
    pub mod iam {
        pub mod v1 {
            include!("../generated/google.iam.v1.rs");
        }
    }
    pub mod r#type {
        include!("../generated/google.type.rs");
    }
    pub mod bigtable {
        pub mod v2 {
            include!("../generated/google.bigtable.v2.rs");
        }

        pub mod admin {
            pub mod v2 {
                include!("../generated/google.bigtable.admin.v2.rs");
            }
        }
    }
}

use api::bigtable::v2;

pub use api::bigtable::v2::ReadRowsRequest;

pub use api::bigtable::v2::{RowRange, RowSet};

fn bound_to_start_key(bound: Bound<&Bytes>) -> Option<v2::row_range::StartKey> {
    use v2::row_range::StartKey;
    match bound {
        Bound::Included(b) => Some(StartKey::StartKeyClosed(b.clone())),
        Bound::Excluded(b) => Some(StartKey::StartKeyOpen(b.clone())),
        Bound::Unbounded => None,
    }
}

fn bound_to_end_key(bound: Bound<&Bytes>) -> Option<v2::row_range::EndKey> {
    use v2::row_range::EndKey;
    match bound {
        Bound::Included(b) => Some(EndKey::EndKeyClosed(b.clone())),
        Bound::Excluded(b) => Some(EndKey::EndKeyOpen(b.clone())),
        Bound::Unbounded => None,
    }
}

/// Returns true if `y` is the smallest `Bytes` that's strictly bigger than `x`.
fn is_succ(x: &Bytes, y: &Bytes) -> bool {
    // The smallest word strictly after "foo" is "fooa".
    y.len() == x.len() + 1 && x[..] == y[..x.len()] && y[x.len()] == 0
}

impl RowRange {
    // Take just the part of this range that's strictly after `key`, returning true if the resulting
    // range is non-empty. (This API is a little weird; the point is that you're supposed to use
    // it in `Vec::retain_mut`.)
    fn restrict_to_after(&mut self, key: &Bytes) -> bool {
        use v2::row_range::{EndKey, StartKey};
        match &self.end_key {
            // (key, b] is non-empty if and only if b > key
            Some(EndKey::EndKeyClosed(b)) if b <= key => return false,
            // (key, b) is non-empty if and only if b > succ(key).
            // The negation of this is `b <= key or b == succ(key)`
            Some(EndKey::EndKeyOpen(b)) if b <= key || is_succ(key, b) => return false,
            _ => {}
        }

        let replace_start_key = match &self.start_key {
            Some(StartKey::StartKeyOpen(b)) if b >= key => false,
            Some(StartKey::StartKeyClosed(b)) if b > key => false,
            _ => true,
        };

        if replace_start_key {
            self.start_key = Some(StartKey::StartKeyOpen(key.clone()));
        }
        true
    }
}

impl RowSet {
    /// Add a range of rows to this row set.
    // TODO: it would be nice to take types more general than Bytes, but the
    // RangeBounds trait makes that annoying: it doesn't allow extracting the
    // start/end by value.
    pub fn with_range(mut self, range: impl RangeBounds<Bytes>) -> Self {
        let start_key = bound_to_start_key(range.start_bound());
        let end_key = bound_to_end_key(range.end_bound());
        let range = v2::RowRange { start_key, end_key };
        self.row_ranges.push(range);
        self
    }

    /// Add a single row to this row set.
    pub fn with_key(mut self, key: impl Into<Bytes>) -> Self {
        self.row_keys.push(key.into());
        self
    }

    fn restrict_to_after(&mut self, key: Bytes) {
        self.row_ranges.retain_mut(|r| r.restrict_to_after(&key));
        self.row_keys.retain(|r| r > &key);
    }
}

/// The default [`RetryPredicate`] used for errors from Bigtable operations
#[derive(Debug, Default, Clone)]
pub struct BigtableRetryCheck {
    _priv: (),
}

impl BigtableRetryCheck {
    /// Create a new instance with default settings
    pub fn new() -> Self {
        Self { _priv: () }
    }
}

impl RetryPredicate<tonic::Status> for BigtableRetryCheck {
    fn is_retriable(&self, error: &tonic::Status) -> bool {
        use tonic::Code;
        // this error code check is based on the ones used in the Go bigtable client lib:
        // https://github.com/googleapis/google-cloud-go/blob/66e8e2717b2593f4e5640ecb97344bb1d5e5fc0b/bigtable/bigtable.go#L107

        match error.code() {
            Code::DeadlineExceeded | Code::Unavailable | Code::Aborted => true,
            _ => false,
        }
    }
}

pub use api::bigtable::v2::{Cell, Column, Family, Row};

/// A reference for a single cell in Bigtable
pub struct CellRef<'a> {
    /// The column family name
    pub family_name: &'a str,
    /// The column qualifier
    pub column_qualifier: &'a [u8],
    /// The cell's timestamp, in microseconds since the epoch
    pub timestamp_micros: i64,
    /// The cell's contents
    pub value: &'a [u8],
}

impl Row {
    /// Iterate over this row's cells, returning only the most recent cell in each column
    pub fn most_recent_cells(&self) -> impl Iterator<Item = CellRef<'_>> + '_ {
        self.families.iter().flat_map(|fam| {
            fam.columns.iter().filter_map(move |col| {
                // The cells vec is *decreasing* in `timestamp_micros`
                col.cells.first().map(move |cell| CellRef {
                    family_name: &fam.name,
                    column_qualifier: &col.qualifier,
                    timestamp_micros: cell.timestamp_micros,
                    value: &cell.value,
                })
            })
        })
    }
}

/// A consistency error when streaming chunks in response to a read request.
/// The best way to recover from these is likely to retry the whole request.
#[derive(Debug, thiserror::Error)]
pub enum ReadInProgressError {
    /// Expected a last cell to exist, but didn't find one.
    #[error("no last cell")]
    NoLastCell,
    /// Expected a last column family to exist, but didn't find one.
    #[error("no last family")]
    NoLastFamily,
    /// Expected a last column to exist, but didn't find one.
    #[error("no last column")]
    NoLastColumn,
}

#[derive(Default)]
struct ReadInProgress {
    row: Row,
    value: BytesMut,
}

impl ReadInProgress {
    fn flush_bytes_in_progress(&mut self) -> Result<(), ReadInProgressError> {
        if self.value.is_empty() {
            Ok(())
        } else {
            let value = self.value.split().freeze();
            if let Some(cell) = self.last_cell_mut() {
                cell.value = value;
                Ok(())
            } else {
                Err(ReadInProgressError::NoLastCell)
            }
        }
    }

    fn new_family(&mut self, name: String) -> Result<(), ReadInProgressError> {
        self.flush_bytes_in_progress()?;
        if self.row.families.last().map(|fam| &fam.name) != Some(&name) {
            self.row.families.push(Family {
                name,
                ..Default::default()
            });
        }
        Ok(())
    }

    fn new_column(&mut self, qualifier: Bytes) -> Result<(), ReadInProgressError> {
        self.flush_bytes_in_progress()?;
        if let Some(family) = self.row.families.last_mut() {
            if family.columns.last().map(|col| &col.qualifier) != Some(&qualifier) {
                family.columns.push(Column {
                    qualifier,
                    ..Default::default()
                });
            }
            Ok(())
        } else {
            Err(ReadInProgressError::NoLastFamily)
        }
    }

    fn last_column_mut(&mut self) -> Option<&mut Column> {
        self.row
            .families
            .last_mut()
            .and_then(|family| family.columns.last_mut())
    }

    fn new_cell(
        &mut self,
        timestamp_micros: i64,
        value_size: i32,
    ) -> Result<(), ReadInProgressError> {
        self.flush_bytes_in_progress()?;
        if let Some(col) = self.last_column_mut() {
            col.cells.push(Cell {
                timestamp_micros,
                ..Default::default()
            });
            self.value = BytesMut::with_capacity(value_size as usize);
            Ok(())
        } else {
            Err(ReadInProgressError::NoLastCell)
        }
    }

    fn last_cell_mut(&mut self) -> Option<&mut Cell> {
        self.last_column_mut().and_then(|col| col.cells.last_mut())
    }

    fn finish_row(&mut self) -> Result<Row, ReadInProgressError> {
        self.flush_bytes_in_progress()?;
        Ok(std::mem::replace(&mut self.row, Default::default()))
    }

    // Process a chunk, and return a row if one was completed.
    //
    // Bigtable responds in chunks, where a row can be split across chunks (but every chunk
    // contains at most one row).
    fn process_chunk(
        &mut self,
        chunk: v2::read_rows_response::CellChunk,
    ) -> Result<Option<Row>, ReadInProgressError> {
        if !chunk.row_key.is_empty() {
            // We don't need to check if there's an existing row to store, because
            // RowStatus tells us when to do that.
            self.row.key = chunk.row_key;
        }

        if let Some(family_name) = chunk.family_name {
            self.new_family(family_name)?;
        }

        if let Some(qualifier) = chunk.qualifier {
            self.new_column(qualifier.into())?;
        }

        if Some(chunk.timestamp_micros) != self.last_cell_mut().map(|cell| cell.timestamp_micros) {
            self.new_cell(chunk.timestamp_micros, chunk.value_size)?;
        }

        // TODO: in the (presumably reasonably common case that the buffer comes
        // in a single chunk, we should avoid copying the data.
        self.value.extend_from_slice(&chunk.value);

        if let Some(row_status) = chunk.row_status {
            let row = self.finish_row();
            match row_status {
                v2::read_rows_response::cell_chunk::RowStatus::CommitRow(_) => row.map(|r| Some(r)),
                // We've already reset our state to an empty row, so to "reset" we just
                // return None.
                v2::read_rows_response::cell_chunk::RowStatus::ResetRow(_) => Ok(None),
            }
        } else {
            Ok(None)
        }
    }
}

/// The error returned from a row read request.
#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
pub enum ReadRowsError {
    /// A gRPC error.
    #[error("gRPC error: {0}")]
    Tonic(tonic::Status),

    /// A consistency error when streaming a row read.
    #[error("row read error")]
    ReadInProgress(ReadInProgressError),
}

impl From<tonic::Status> for ReadRowsError {
    fn from(s: tonic::Status) -> Self {
        Self::Tonic(s)
    }
}

impl From<ReadInProgressError> for ReadRowsError {
    fn from(e: ReadInProgressError) -> Self {
        Self::ReadInProgress(e)
    }
}

/// A client for connecting to bigtable. Created from the
/// [`build_bigtable_client`](crate::builder::ClientBuilder::build_bigtable_client)
/// function.
#[derive(Clone)]
pub struct BigtableClient<
    Service = AuthGrpcService<tonic::transport::Channel, crate::DefaultConnector>,
    Retry = ExponentialBackoff<BigtableRetryCheck>,
> {
    inner: api::bigtable::v2::bigtable_client::BigtableClient<Service>,
    retry: Retry,
    table_prefix: String,
}

impl<Service> BigtableClient<Service>
where
    Service: tonic::client::GrpcService<tonic::body::BoxBody>,
    Service::ResponseBody: http_body::Body + Send + 'static,
    Service::Error: Into<Box<dyn std::error::Error + Send + Sync + 'static>>,
    <Service::ResponseBody as http_body::Body>::Error:
        Into<Box<dyn std::error::Error + Send + Sync + 'static>> + Send,
{
    /// Create a `BigtableClient` using a provided [`GrpcService`] for communicating
    /// to bigtable.
    ///
    /// To connect to a non-emulated bigtable, the service will need to provide authentication;
    /// [`AuthGrpcService`] is a wrapper that adds authentication to an unauthenticated service.
    ///
    /// # Example
    /// ```
    /// # use ya_gcp::bigtable::{BigtableConfig, BigtableClient};
    /// # use ya_gcp::auth::grpc::AuthGrpcService;
    /// # use hyper::body::Bytes;
    /// # let _ = || async move {
    /// // Set up auth using yup_oauth2. In this example, we grab it from the user credentials created
    /// // by `gloud auth application-default login`.
    /// let secret = yup_oauth2::read_authorized_user_secret("/home/me/.config/gcloud/application_default_credentials.json").await?;
    /// let auth = yup_oauth2::AuthorizedUserAuthenticator::builder(secret).build().await?;
    ///
    /// let config = BigtableConfig::default();
    /// let channel = tonic::transport::Endpoint::from_shared(config.endpoint.clone())?.connect().await?;
    /// let auth_channel = AuthGrpcService::new(channel, Some(auth), config.auth_scopes());
    /// let client = BigtableClient::new(auth_channel, "my-project", "my-instance");
    /// # Ok::<_, Box<dyn std::error::Error>>(())
    /// # };
    /// ```
    ///
    /// [`GrpcService`]: tonic::client::GrpcService
    pub fn new(service: Service, project: &str, instance_name: &str) -> Self {
        let retry = ExponentialBackoff::new(
            BigtableRetryCheck::default(),
            exponential_backoff::Config::default(),
        );
        Self::with_retry(service, retry, project, instance_name)
    }
}

impl<Service, Retry> BigtableClient<Service, Retry>
where
    Service: tonic::client::GrpcService<tonic::body::BoxBody>,
    Service::ResponseBody: http_body::Body + Send + 'static,
    Service::Error: Into<Box<dyn std::error::Error + Send + Sync + 'static>>,
    <Service::ResponseBody as http_body::Body>::Error:
        Into<Box<dyn std::error::Error + Send + Sync + 'static>> + Send,
    Retry: RetryPolicy<(), tonic::Status> + 'static,
    Retry::RetryOp: Send + 'static,
    <Retry::RetryOp as RetryOperation<(), tonic::Status>>::Sleep: Send + 'static,
{
    /// Create a `BigtableClient` with a custom retry policy and using a
    /// provided [`GrpcService`] for communicating to bigtable.
    ///
    /// To connect to a non-emulated bigtable, the service will need to provide authentication;
    /// [`AuthGrpcService`] is a wrapper that adds authentication to an unauthenticated service.
    ///
    /// [`GrpcService`]: tonic::client::GrpcService
    pub fn with_retry(service: Service, retry: Retry, project: &str, instance_name: &str) -> Self {
        let table_prefix = format!("projects/{}/instances/{}/tables/", project, instance_name);
        let inner = api::bigtable::v2::bigtable_client::BigtableClient::new(service);

        BigtableClient {
            inner,
            table_prefix,
            retry,
        }
    }

    /// Request some rows from bigtable.
    ///
    /// This is the most general read request; various other convenience methods are
    /// available.
    /// Note that the table name in `request` needs to be
    /// [fully qualified](Self::fully_qualified_table_name).
    pub fn read_rows(
        &mut self,
        mut request: ReadRowsRequest,
    ) -> impl Stream<Item = Result<Row, ReadRowsError>> + '_ {
        async_stream::stream! {
            let mut retry = self.retry.new_operation();

            'retry: loop {
                // Keep track of the last returned (and/or last scanned) key, so that we won't
                // request it again if we need to retry.
                let mut last_key: Option<Bytes> = None;

                let mut state = ReadInProgress::default();
                let mut response = self.inner.read_rows(request.clone()).await?.into_inner();
                let result = 'response_part: loop {
                    let message = match response.next().await {
                        Some(m) => m,
                        None => break 'response_part Ok(()),
                    };

                    let message = match message {
                        Ok(m) => m,
                        Err(e) => break 'response_part Err(e),
                    };

                    last_key = Some(
                        last_key
                            .unwrap_or_default()
                            .max(message.last_scanned_row_key),
                    );

                    for chunk in message.chunks {
                        if let Some(row) = state.process_chunk(chunk)? {
                            last_key = Some(last_key.unwrap_or_default().max(row.key.clone()));
                            yield Ok(row);
                        }
                    }
                };

                if let Err(e) = result {
                    if let Some(sleep) = retry.check_retry(&(), &e) {
                        sleep.await;
                        // Loop back and retry the request again, but
                        // first modify the request to avoid re-requesting
                        // previously-returned data.
                        if let Some(key) = last_key {
                            match request.rows.as_mut() {
                                Some(r) => r.restrict_to_after(key),
                                // If the request had no rows then we were trying to read a whole table.
                                // Since we have a `last_key` we limit the retry to everything *after*
                                // that key, in order to avoid re-reading rows we've already covered.
                                None => {
                                    request.rows = Some(RowSet {
                                        row_ranges: vec![
                                            RowRange {
                                                start_key: Some(v2::row_range::StartKey::StartKeyOpen(key)),
                                                end_key: None
                                            }
                                        ],
                                        ..Default::default()
                                    })
                                },
                            };
                        }
                        continue 'retry;
                    } else {
                        yield Err(e.into());
                        return;
                    }
                } else {
                    return;
                }
            }
        }
    }

    /// Queries a range of rows from a table, returning just the keys.
    pub fn read_row_range_keys(
        &mut self,
        table_name: &str,
        range: impl RangeBounds<Bytes>,
        rows_limit: Option<i64>,
    ) -> impl Stream<Item = Result<Bytes, ReadRowsError>> + '_ {
        use filters::{Chain, Filter};
        let req = ReadRowsRequest {
            table_name: self.fully_qualified_table_name(table_name),
            rows_limit: rows_limit.unwrap_or(0),
            rows: Some(v2::RowSet::default().with_range(range)),
            filter: Some(
                Chain::default()
                    // Return the minimal number of cells per row
                    .with_filter(Filter::CellsPerRowLimitFilter(1))
                    // Return the minimal number of cells per column
                    .with_filter(Filter::CellsPerColumnLimitFilter(1))
                    // Strip off the cell data
                    .with_filter(Filter::StripValueTransformer(true))
                    .into(),
            ),
            ..Default::default()
        };
        self.read_rows(req)
            .map(|maybe_row| maybe_row.map(|row| row.key))
    }

    /// Queries a range of rows from a table, returning only the most recent value of each cell.
    pub fn read_row_range(
        &mut self,
        table_name: &str,
        range: impl RangeBounds<Bytes>,
        rows_limit: Option<i64>,
    ) -> impl Stream<Item = Result<Row, ReadRowsError>> + '_ {
        let req = ReadRowsRequest {
            table_name: self.fully_qualified_table_name(table_name),
            rows_limit: rows_limit.unwrap_or(0),
            rows: Some(v2::RowSet::default().with_range(range)),
            filter: Some(filters::Filter::CellsPerColumnLimitFilter(1).into()),
            ..Default::default()
        };
        self.read_rows(req)
    }

    /// Queries a single row from a table, returning on the most recent value of each cell.
    pub async fn read_one_row(
        &mut self,
        table_name: &str,
        row_key: impl Into<Bytes>,
    ) -> Result<Option<Row>, ReadRowsError> {
        let req = ReadRowsRequest {
            table_name: self.fully_qualified_table_name(table_name),
            rows: Some(v2::RowSet::default().with_key(row_key)),
            filter: Some(filters::Filter::CellsPerColumnLimitFilter(1).into()),
            ..Default::default()
        };
        let stream = self.read_rows(req);
        Box::pin(stream).next().await.transpose()
    }

    /// Performs a batch mutation request.
    ///
    /// This is the most general mutation request; various convenience methods are also available.
    /// Note that the table name in `request` needs to be
    /// [fully qualified](Self::fully_qualified_table_name).
    pub async fn mutate_rows(&mut self, request: MutateRowsRequest) -> Result<(), MutateRowsError> {
        let mut retry = self.retry.new_operation();

        let mut response = loop {
            match self.inner.mutate_rows(request.clone()).await {
                Ok(resp) => {
                    break resp.into_inner();
                }
                Err(e) => {
                    if let Some(sleep) = retry.check_retry(&(), &e) {
                        sleep.await;
                    } else {
                        return Err(e.into());
                    }
                }
            }
        };

        // Collect all entries in all responses with a bad status code.
        let mut errors = Vec::new();
        while let Some(res) = response.message().await? {
            errors.extend(res.entries.into_iter().filter(|entry| {
                entry.status.as_ref().map(|status| status.code != 0) == Some(true)
            }));
        }

        if !errors.is_empty() {
            Err(errors.into())
        } else {
            Ok(())
        }
    }

    /// Performs a mutation request for a single row.
    ///
    /// Note that the table name in `request` needs to be
    /// [fully qualified](Self::fully_qualified_table_name).
    pub async fn mutate_row(&mut self, request: MutateRowRequest) -> Result<(), tonic::Status> {
        let mut retry = self.retry.new_operation();

        while let Err(e) = self.inner.mutate_row(request.clone()).await {
            if let Some(sleep) = retry.check_retry(&(), &e) {
                sleep.await;
            } else {
                return Err(e);
            }
        }

        Ok(())
    }

    /// Delete one or more rows from the table.
    pub async fn delete_rows(
        &mut self,
        table_name: &str,
        row_keys: impl IntoIterator<Item = impl Into<Bytes>>,
    ) -> Result<(), MutateRowsError> {
        let table_name = self.fully_qualified_table_name(table_name);
        let req =
            MutateRowsRequest::new(table_name).with_entries(row_keys.into_iter().map(|row_key| {
                mutation::Entry::new(row_key.into()).with_mutation(mutation::DeleteFromRow {})
            }));
        self.mutate_rows(req).await
    }

    /// Set some data for a single row.
    pub async fn set_row_data<RowKey, RowData, ColName, CellData>(
        &mut self,
        table_name: &str,
        family_name: String,
        row_key: RowKey,
        data: RowData,
    ) -> Result<(), tonic::Status>
    where
        RowKey: Into<Bytes>,
        ColName: Into<Bytes>,
        CellData: Into<Bytes>,
        RowData: IntoIterator<Item = (ColName, CellData)>,
    {
        self.set_row_data_with_timestamp(table_name, family_name, -1, row_key, data)
            .await
    }

    /// Set data for a collection of rows, all having the same table name and column family name.
    ///
    /// If `timestamp` is `None`, the bigtable server will be in charge of choosing the timestamp.
    /// For more on the purpose of the `timestamp` field, see
    /// [Google's docs](https://cloud.google.com/bigtable/docs/garbage-collection#timestamps).
    pub async fn set_rows_data<RowKey, RowData, ColName, CellData>(
        &mut self,
        table_name: &str,
        family_name: String,
        timestamp: Option<i64>,
        data: impl IntoIterator<Item = (RowKey, RowData)>,
    ) -> Result<(), MutateRowsError>
    where
        RowKey: Into<Bytes>,
        ColName: Into<Bytes>,
        CellData: Into<Bytes>,
        RowData: IntoIterator<Item = (ColName, CellData)>,
    {
        let table_name = self.fully_qualified_table_name(table_name);
        let req = MutateRowsRequest::new(table_name).with_entries(data.into_iter().map(
            |(row_key, row_data)| {
                mutation::Entry::new(row_key.into()).with_mutations(row_data.into_iter().map(
                    |(col_name, cell_data)| {
                        mutation::SetCell::new(
                            family_name.clone(),
                            col_name.into(),
                            cell_data.into(),
                        )
                        .with_timestamp(timestamp.unwrap_or(-1))
                    },
                ))
            },
        ));

        self.mutate_rows(req).await
    }

    /// Set some data for a single row.
    ///
    /// For more on the purpose of the `timestamp` field, see
    /// [Google's docs](https://cloud.google.com/bigtable/docs/garbage-collection#timestamps).
    pub async fn set_row_data_with_timestamp<RowKey, RowData, ColName, CellData>(
        &mut self,
        table_name: &str,
        family_name: String,
        timestamp: i64,
        row_key: RowKey,
        data: RowData,
    ) -> Result<(), tonic::Status>
    where
        RowKey: Into<Bytes>,
        ColName: Into<Bytes>,
        CellData: Into<Bytes>,
        RowData: IntoIterator<Item = (ColName, CellData)>,
    {
        let table_name = self.fully_qualified_table_name(table_name);
        let req = MutateRowRequest::new(table_name, row_key.into()).with_mutations(
            data.into_iter().map(|(col_name, cell_data)| {
                mutation::SetCell::new(family_name.clone(), col_name.into(), cell_data.into())
                    .with_timestamp(timestamp)
            }),
        );
        self.mutate_row(req).await
    }

    /// Builds a fully qualified table name, of the form `projects/<PROJECT>/instances/<INSTANCE>/tables/<TABLE>`.
    ///
    /// If you are building your own [`ReadRowsRequest`], [`MutateRowRequest`], or [`MutateRowsRequest`], you need to provide
    /// this fully qualified table name. If you are using one of the other convenience query functions,
    /// you do not need the fully qualified table name.
    pub fn fully_qualified_table_name(&self, table_name: &str) -> String {
        format!("{}{}", self.table_prefix, table_name)
    }
}
