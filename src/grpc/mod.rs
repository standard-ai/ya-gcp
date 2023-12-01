//! Items and functionality specific to gRPC services

pub use tonic::{
    body::BoxBody,
    client::GrpcService,
    codegen::{Body, Bytes, StdError},
};

mod status_code_set;
pub use status_code_set::StatusCodeSet;

/// The default grpc transport implementation
pub type DefaultGrpcImpl = crate::auth::grpc::AuthGrpcService<tonic::transport::Channel>;
