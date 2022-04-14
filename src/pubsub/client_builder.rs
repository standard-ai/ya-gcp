use crate::{
    auth::grpc,
    builder,
    pubsub::{api, PublisherClient, SubscriberClient},
};
use std::time::Duration;

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
        @default("https://pubsub.googleapis.com/v1".into(), "PubSubConfig::default_endpoint")
        pub endpoint: String,

        /// The authorization scopes to use when requesting auth tokens
        @default(vec!["https://www.googleapis.com/auth/pubsub".into()], "PubSubConfig::default_auth_scopes")
        pub auth_scopes: Vec<String>,

        /// The timeout to apply to publish requests
        //TODO(major semver) move this into dedicated PublishConfig
        @default(Duration::from_secs(60), "PubSubConfig::default_publish_timeout")
        pub publish_timeout: Duration,
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
        endpoint: tonic::transport::Endpoint,
        auth_scopes: Vec<String>,
    ) -> Result<
        grpc::AuthGrpcService<tonic::transport::Channel, grpc::OAuthTokenSource<C>>,
        BuildError,
    > {
        let connection = endpoint
            .connect_with_connector(self.connector.clone())
            .await?;

        Ok(grpc::oauth_grpc(connection, self.auth.clone(), auth_scopes))
    }

    /// Create a client for publishing to the pubsub service
    pub async fn build_pubsub_publisher(
        &self,
        config: PubSubConfig,
    ) -> Result<PublisherClient<C>, BuildError> {
        // TODO this is actually too high of a level to apply timeouts. For one thing, it's applied
        // to every request from a publisher (including stuff like topic creation), not just
        // publish requests; yet it's not applied to every service layer, for example auth requests
        // during publishing aren't covered.
        //
        // This should instead probably be applied within the publisher sink itself, around the
        // publish call. However it's hard to sneak a timeout in there because the sink is
        // technically runtime agnostic, and timers in runtime agnostic code are a pain.
        //
        // TODO(major semver) perhaps break the facade of agnosticism, embrace tokio, and put
        // publish timeouts at the right layer?
        let endpoint =
            tonic::transport::Endpoint::new(config.endpoint)?.timeout(config.publish_timeout);

        // the crate's client will wrap the raw grpc client to add features/functions/ergonomics
        Ok(PublisherClient {
            inner: api::publisher_client::PublisherClient::new(
                self.pubsub_authed_service(endpoint, config.auth_scopes)
                    .await?,
            ),
        })
    }

    /// Create a client for subscribing to the pubsub service
    pub async fn build_pubsub_subscriber(
        &self,
        config: PubSubConfig,
    ) -> Result<SubscriberClient<C>, BuildError> {
        let endpoint = tonic::transport::Endpoint::new(config.endpoint)?;

        // the crate's client will wrap the raw grpc client to add features/functions/ergonomics
        Ok(SubscriberClient {
            inner: api::subscriber_client::SubscriberClient::new(
                self.pubsub_authed_service(endpoint, config.auth_scopes)
                    .await?,
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
        assert_eq!(config.endpoint, "https://pubsub.googleapis.com/v1");
    }
}
