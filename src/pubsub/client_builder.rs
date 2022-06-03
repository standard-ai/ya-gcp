use crate::{
    auth::grpc,
    builder,
    grpc::EndpointConfig,
    pubsub::{api, PublisherClient, SubscriberClient},
};
use std::{convert::TryFrom, time::Duration};

// re-export traits and types necessary for the bounds on public functions
#[allow(unreachable_pub)] // the reachability lint seems faulty with parent module re-exports
pub use http::Uri;
#[allow(unreachable_pub)]
pub use tower::make::MakeConnection;

config_default! {
    /// Configuration for connecting to pubsub
    #[derive(Debug, Clone, Eq, PartialEq, Hash, serde::Deserialize)]
    #[non_exhaustive]
    pub struct PubSubConfig {
        /// Endpoint to connect to pubsub over.
        #[serde(with = "UriDeser")]
        @default(Uri::from_static("https://pubsub.googleapis.com:443"), "PubSubConfig::default_endpoint")
        pub endpoint: Uri,

        /// The authorization scopes to use when requesting auth tokens
        @default(
            vec![
                "https://www.googleapis.com/auth/cloud-platform".into(),
                "https://www.googleapis.com/auth/pubsub".into(),
            ],
            "PubSubConfig::default_auth_scopes"
        )
        pub auth_scopes: Vec<String>,

        /// Configuration for the connections to the endpoint
        @default(
            EndpointConfig::default()
                // java and go set a keep alive interval of 5 minutes
                // https://github.com/googleapis/java-pubsub/blob/3a8c83b973a1dfbae2ca037125574d74034218ce/google-cloud-pubsub/src/main/java/com/google/cloud/pubsub/v1/Subscriber.java#L487
                // https://github.com/googleapis/google-cloud-go/blob/pubsub/v1.21.0/pubsub/pubsub.go#L144
                .http2_keep_alive_interval(Some(Duration::from_secs(5 * 60))),
            "PubSubConfig::default_endpoint_config"
        )
        pub endpoint_config: EndpointConfig,

        /// The number of underlying connections to the gRPC servers. Requests will be load
        /// balanced across these connections
        // go sets a pool size of min(GOMAXPROCS, 4)
        // java uses a complex adaptive pool size
        // given this is configurable, just set to 4 and let the user decide
        // https://github.com/googleapis/google-cloud-go/blob/pubsub/v1.21.0/pubsub/pubsub.go#L138
        // https://github.com/googleapis/gax-java/blob/fff2babaf2620686c2e0be1b6d338ac088248cf6/gax-grpc/src/main/java/com/google/api/gax/grpc/ChannelPoolSettings.java#L139-L141
        @default(4, "PubSubConfig::default_connection_pool_size")
        pub connection_pool_size: usize,
    }
}

// Stub to deserialize a Uri from String
#[derive(serde::Deserialize)]
#[serde(try_from = "String", remote = "Uri")]
struct UriDeser(Uri);

impl TryFrom<String> for UriDeser {
    type Error = <Uri as TryFrom<String>>::Error;
    fn try_from(string: String) -> Result<Self, Self::Error> {
        Uri::try_from(string).map(Self)
    }
}

impl From<UriDeser> for Uri {
    fn from(UriDeser(uri): UriDeser) -> Self {
        uri
    }
}

/// An error encountered when building PubSub clients
#[derive(Debug, thiserror::Error)]
#[error(transparent)]
pub struct BuildError(#[from] tonic::transport::Error);

impl<C> builder::ClientBuilder<C>
where
    C: MakeConnection<Uri> + crate::Connect + Clone + Send + Sync + 'static,
    C::Connection: Unpin + Send + 'static,
    C::Future: Send + 'static,
    Box<dyn std::error::Error + Send + Sync + 'static>: From<C::Error>,
{
    async fn pubsub_authed_service(
        &self,
        config: PubSubConfig,
    ) -> Result<
        grpc::AuthGrpcService<tonic::transport::Channel, grpc::OAuthTokenSource<C>>,
        BuildError,
    > {
        let endpoint = tonic::transport::Endpoint::new(config.endpoint)?;
        let endpoint = config.endpoint_config.apply(endpoint);

        //TODO technically a breaking change, no longer delegates to user connector
        let connection = tonic::transport::Channel::balance_list(
            std::iter::repeat(endpoint).take(usize::min(config.connection_pool_size, 1)),
        );

        Ok(grpc::oauth_grpc(
            connection,
            self.auth.clone(),
            config.auth_scopes,
        ))
    }

    /// Create a client for publishing to the pubsub service
    pub async fn build_pubsub_publisher(
        &self,
        config: PubSubConfig,
    ) -> Result<PublisherClient<C>, BuildError> {
        // the crate's client will wrap the raw grpc client to add features/functions/ergonomics
        Ok(PublisherClient {
            inner: api::publisher_client::PublisherClient::new(
                self.pubsub_authed_service(config).await?,
            ),
        })
    }

    /// Create a client for subscribing to the pubsub service
    pub async fn build_pubsub_subscriber(
        &self,
        config: PubSubConfig,
    ) -> Result<SubscriberClient<C>, BuildError> {
        // the crate's client will wrap the raw grpc client to add features/functions/ergonomics
        Ok(SubscriberClient {
            inner: api::subscriber_client::SubscriberClient::new(
                self.pubsub_authed_service(config).await?,
            ),
        })
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn config_default() {
        let config = PubSubConfig::default();
        assert_eq!(config.endpoint, "https://pubsub.googleapis.com:443/");
    }
}
