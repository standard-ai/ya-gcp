use crate::{
    bigtable::{api, BigtableClient},
    builder, grpc,
};

const BIGTABLE_DATA_SCOPE: &'static str = "https://www.googleapis.com/auth/bigtable.data";
const BIGTABLE_DATA_READONLY_SCOPE: &'static str =
    "https://www.googleapis.com/auth/bigtable.data.readonly";

// I couldn't find any documentation on the maximum size for bigtable replies,
// but this suggests it can be large:
// https://github.com/googleapis/google-cloud-node/issues/1755
const MAX_MESSAGE_SIZE: usize = usize::MAX;

config_default! {
    /// Configuration for connecting to bigtable
    #[derive(Debug, Clone, Eq, PartialEq, Hash, serde::Deserialize)]
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
    fn auth_scopes(&self) -> Vec<String> {
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

impl builder::ClientBuilder {
    /// Create a client for connecting to bigtable
    pub async fn build_bigtable_client(
        &self,
        config: BigtableConfig,
        project: &str,
        instance_name: &str,
    ) -> Result<BigtableClient<grpc::DefaultGrpcImpl>, BuildError> {
        let scopes = config.auth_scopes();
        let endpoint = tonic::transport::Endpoint::new(config.endpoint)?;

        let connection = endpoint.connect().await?;

        let inner = api::bigtable::v2::bigtable_client::BigtableClient::new(
            grpc::DefaultGrpcImpl::new(connection, self.auth.clone(), scopes),
        )
        .max_decoding_message_size(MAX_MESSAGE_SIZE);

        Ok(BigtableClient::from_raw_api(inner, project, instance_name))
    }
}
