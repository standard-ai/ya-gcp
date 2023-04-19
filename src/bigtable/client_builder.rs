use crate::{
    auth::grpc::{self, AuthGrpcService},
    bigtable::{api, BigtableClient},
    builder,
    retry_policy::{exponential_backoff, ExponentialBackoff},
};

const BIGTABLE_DATA_SCOPE: &'static str = "https://www.googleapis.com/auth/bigtable.data";
const BIGTABLE_DATA_READONLY_SCOPE: &'static str =
    "https://www.googleapis.com/auth/bigtable.data.readonly";

config_default! {
    /// Configuration for connecting to bigtable
    #[derive(Debug, Clone, Eq, PartialEq, Hash, serde::Deserialize)]
    #[non_exhaustive]
    pub struct BigtableConfig {
        /// Endpoint to connect to bigtable over.
        @default("https://bigtable.googleapis.com".into(), "BigtableConfig::default_endpoint")
        pub endpoint: String,

        /// Whether this client should be created with only read permission.
        @default(false, "BigtableConfig::default_readonly")
        pub readonly: bool,
    }
}

impl BigtableConfig {
    /// Returns the oauth scopes required for this bigtable configuration.
    ///
    /// These are handled automatically when using [`build_bigtable_client`],
    /// but you will need them if you intend to provide your own authentication.
    ///
    /// [`build_bigtable_client`]: crate::ClientBuilder::build_bigtable_client
    pub fn auth_scopes(&self) -> Vec<String> {
        if self.readonly {
            vec![BIGTABLE_DATA_READONLY_SCOPE.to_owned()]
        } else {
            vec![BIGTABLE_DATA_SCOPE.to_owned()]
        }
    }
}

/// An error encountered when building Bigtable clients
#[derive(Debug, thiserror::Error)]
#[error(transparent)]
pub struct BuildError(#[from] tonic::transport::Error);

use super::BigtableRetryCheck;

impl<C> builder::ClientBuilder<C>
where
    C: tower::Service<http::Uri> + Clone + Send + Sync + 'static,
    C::Response: hyper::client::connect::Connection
        + tokio::io::AsyncRead
        + tokio::io::AsyncWrite
        + Send
        + Unpin
        + 'static,
    C::Future: Send + Unpin + 'static,
    C::Error: Into<Box<dyn std::error::Error + Send + Sync>>,
    Box<dyn std::error::Error + Send + Sync + 'static>: From<C::Error>,
{
    /// Create a client for connecting to bigtable
    pub async fn build_bigtable_client(
        &self,
        config: BigtableConfig,
        project: &str,
        instance_name: &str,
    ) -> Result<BigtableClient<AuthGrpcService<tonic::transport::Channel, C>>, BuildError> {
        let scopes = config.auth_scopes();
        let endpoint = tonic::transport::Endpoint::new(config.endpoint)?;

        let connection = endpoint
            .connect_with_connector(self.connector.clone())
            .await?;
        let table_prefix = format!("projects/{}/instances/{}/tables/", project, instance_name);

        let inner = api::bigtable::v2::bigtable_client::BigtableClient::new(
            grpc::AuthGrpcService::new(connection, self.auth.clone(), scopes),
        );

        Ok(BigtableClient {
            inner,
            table_prefix,
            retry: ExponentialBackoff::new(
                BigtableRetryCheck::default(),
                exponential_backoff::Config::default(),
            ),
        })
    }
}
