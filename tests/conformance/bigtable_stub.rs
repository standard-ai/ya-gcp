use std::pin::Pin;

use futures::{stream, Stream, StreamExt};

#[allow(rustdoc::broken_intra_doc_links, rustdoc::bare_urls, missing_docs)]
pub mod api {
    pub mod rpc {
        include!("generated/google.rpc.rs");
        include!("generated/google.rpc.serde.rs");
    }

    pub mod bigtable {
        pub mod v2 {
            include!("generated/google.bigtable.v2.rs");
            include!("generated/google.bigtable.v2.serde.rs");
        }
    }
}

use api::bigtable::v2 as bigtable;
use api::bigtable::v2::bigtable_server;

#[derive(Clone)]
pub struct StreamStub<Item> {
    inner: stream::Iter<std::vec::IntoIter<Item>>,
}

impl<Item> StreamStub<Item> {
    pub fn new(responses: Vec<Item>) -> Self {
        let inner = stream::iter(responses);
        Self { inner }
    }
}

impl<Item> Stream for StreamStub<Item> {
    type Item = Item;

    fn poll_next(
        mut self: Pin<&mut Self>,
        ctx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Option<Item>> {
        self.inner.poll_next_unpin(ctx)
    }
}

type StubBigtableStream<T> = StreamStub<Result<T, tonic::Status>>;

pub struct StubBigtableServer {
    read_rows_stub: Option<StubBigtableStream<bigtable::ReadRowsResponse>>,
}

impl StubBigtableServer {
    pub fn new() -> Self {
        Self {
            read_rows_stub: None,
        }
    }

    pub fn stub_read_rows(
        &mut self,
        stub: StreamStub<Result<bigtable::ReadRowsResponse, tonic::Status>>,
    ) {
        self.read_rows_stub = Some(stub);
    }
}

#[tonic::async_trait]
impl bigtable_server::Bigtable for StubBigtableServer {
    type ReadRowsStream = StubBigtableStream<bigtable::ReadRowsResponse>;

    async fn read_rows(
        &self,
        _request: tonic::Request<bigtable::ReadRowsRequest>,
    ) -> Result<tonic::Response<Self::ReadRowsStream>, tonic::Status> {
        if let Some(stub) = &self.read_rows_stub {
            Ok(tonic::Response::new(stub.clone()))
        } else {
            Err(tonic::Status::unimplemented("not implemented"))
        }
    }

    type SampleRowKeysStream = StubBigtableStream<bigtable::SampleRowKeysResponse>;

    async fn sample_row_keys(
        &self,
        _request: tonic::Request<bigtable::SampleRowKeysRequest>,
    ) -> Result<tonic::Response<Self::SampleRowKeysStream>, tonic::Status> {
        Err(tonic::Status::unimplemented("not implemented"))
    }

    async fn mutate_row(
        &self,
        _request: tonic::Request<bigtable::MutateRowRequest>,
    ) -> Result<tonic::Response<bigtable::MutateRowResponse>, tonic::Status> {
        Err(tonic::Status::unimplemented("not implemented"))
    }

    type MutateRowsStream = StubBigtableStream<bigtable::MutateRowsResponse>;

    async fn mutate_rows(
        &self,
        _request: tonic::Request<bigtable::MutateRowsRequest>,
    ) -> Result<tonic::Response<Self::MutateRowsStream>, tonic::Status> {
        Err(tonic::Status::unimplemented("not implemented"))
    }

    async fn check_and_mutate_row(
        &self,
        _request: tonic::Request<bigtable::CheckAndMutateRowRequest>,
    ) -> Result<tonic::Response<bigtable::CheckAndMutateRowResponse>, tonic::Status> {
        Err(tonic::Status::unimplemented("not implemented"))
    }

    async fn ping_and_warm(
        &self,
        _request: tonic::Request<bigtable::PingAndWarmRequest>,
    ) -> Result<tonic::Response<bigtable::PingAndWarmResponse>, tonic::Status> {
        Err(tonic::Status::unimplemented("not implemented"))
    }

    async fn read_modify_write_row(
        &self,
        _request: tonic::Request<bigtable::ReadModifyWriteRowRequest>,
    ) -> Result<tonic::Response<bigtable::ReadModifyWriteRowResponse>, tonic::Status> {
        Err(tonic::Status::unimplemented("not implemented"))
    }

    type GenerateInitialChangeStreamPartitionsStream =
        StubBigtableStream<bigtable::GenerateInitialChangeStreamPartitionsResponse>;

    async fn generate_initial_change_stream_partitions(
        &self,
        _request: tonic::Request<bigtable::GenerateInitialChangeStreamPartitionsRequest>,
    ) -> Result<tonic::Response<Self::GenerateInitialChangeStreamPartitionsStream>, tonic::Status>
    {
        Err(tonic::Status::unimplemented("not implemented"))
    }

    type ReadChangeStreamStream = StubBigtableStream<bigtable::ReadChangeStreamResponse>;

    async fn read_change_stream(
        &self,
        _request: tonic::Request<bigtable::ReadChangeStreamRequest>,
    ) -> Result<tonic::Response<Self::ReadChangeStreamStream>, tonic::Status> {
        Err(tonic::Status::unimplemented("not implemented"))
    }
}
