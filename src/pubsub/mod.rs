//! An API for interacting with the [Pub/Sub](https://cloud.google.com/pubsub) message service.
//!
//! Publishing and topic management is done through the [`PublisherClient`], while reading data and
//! subscription management is done through the [`SubscriberClient`].

use crate::grpc::{Body, BoxBody, Bytes, DefaultGrpcImpl, GrpcService, StdError};
use crate::retry_policy::RetryPredicate;
use std::fmt::Display;
use tracing::debug_span;

// alias Status as this module's error type
pub use tonic::Status as Error;

pub use client_builder::{BuildError, MakeConnection, PubSubConfig, Uri};
pub use publish_sink::{PublishConfig, PublishError, PublishTopicSink, SinkError};
pub use streaming_subscription::{
    AcknowledgeError, AcknowledgeToken, ModifyAcknowledgeError, StreamSubscription,
    StreamSubscriptionConfig,
};

pub(crate) mod client_builder;
mod publish_sink;
mod streaming_subscription;

#[cfg(feature = "emulators")]
#[cfg_attr(docsrs, doc(cfg(feature = "emulators")))]
pub mod emulator;

/// Types and functions generated from PubSub's gRPC schema
#[allow(
    rustdoc::broken_intra_doc_links,
    rustdoc::bare_urls,
    missing_docs,
    unreachable_pub
)]
pub mod api {
    include!("../generated/google.pubsub.v1.rs");

    // re-exports of prost types used within the generated code for convenience
    pub use prost_types::{Duration, FieldMask, Timestamp};
}

/// A client through which pubsub messages are sent, and topics are managed. Created
/// from the [`build_pubsub_publisher`](crate::builder::ClientBuilder::build_pubsub_publisher)
/// function.
///
/// This builds on top of the raw [gRPC publisher API](api::publisher_client::PublisherClient)
/// to provide more ergonomic functionality
#[derive(Debug, Clone)]
pub struct PublisherClient<S = DefaultGrpcImpl> {
    inner: api::publisher_client::PublisherClient<S>,
}

impl<S> PublisherClient<S> {
    /// Manually construct a new client.
    ///
    /// There are limited circumstances in which this is useful; consider instead using the builder
    /// function [crate::builder::ClientBuilder::build_pubsub_publisher]
    pub fn from_raw_api(client: api::publisher_client::PublisherClient<S>) -> Self {
        PublisherClient { inner: client }
    }

    /// Access the underlying grpc api
    pub fn raw_api(&self) -> &api::publisher_client::PublisherClient<S> {
        &self.inner
    }

    /// Mutably access the underlying grpc api
    pub fn raw_api_mut(&mut self) -> &mut api::publisher_client::PublisherClient<S> {
        &mut self.inner
    }
}

impl<S> PublisherClient<S>
where
    S: GrpcService<BoxBody> + Clone,
    S::Error: Into<StdError>,
    S::ResponseBody: Body<Data = Bytes> + Send + 'static,
    <S::ResponseBody as Body>::Error: Into<StdError> + Send,
{
    /// Create a sink which will publish [messages](api::PubsubMessage) to the given topic.
    ///
    /// See the type's [documentation](PublishTopicSink) for more details.
    pub fn publish_topic_sink(
        &mut self,
        topic: ProjectTopicName,
        config: PublishConfig,
    ) -> PublishTopicSink<S> {
        PublishTopicSink::new(self.inner.clone(), topic, config)
    }
}

/// A client through which pubsub messages are consumed, and subscriptions are managed. Created
/// from the [`build_pubsub_subscriber`](crate::builder::ClientBuilder::build_pubsub_subscriber)
/// function.
///
/// This is an interface built on top of the raw [gRPC subscriber
/// API](api::subscriber_client::SubscriberClient) which provides more ergonomic functionality
#[derive(Debug, Clone)]
pub struct SubscriberClient<S = DefaultGrpcImpl> {
    inner: api::subscriber_client::SubscriberClient<S>,
}

impl<S> SubscriberClient<S> {
    /// Manually construct a new client.
    ///
    /// There are limited circumstances in which this is useful; consider instead using the builder
    /// function [crate::builder::ClientBuilder::build_pubsub_subscriber]
    pub fn from_raw_api(client: api::subscriber_client::SubscriberClient<S>) -> Self {
        Self { inner: client }
    }

    /// Access the underlying grpc api
    pub fn raw_api(&self) -> &api::subscriber_client::SubscriberClient<S> {
        &self.inner
    }

    /// Mutably access the underlying grpc api
    pub fn raw_api_mut(&mut self) -> &mut api::subscriber_client::SubscriberClient<S> {
        &mut self.inner
    }
}

impl<S> SubscriberClient<S>
where
    S: GrpcService<BoxBody> + Clone,
    S::Error: Into<StdError>,
    S::ResponseBody: Body<Data = Bytes> + Send + 'static,
    <S::ResponseBody as Body>::Error: Into<StdError> + Send,
{
    /// Start a streaming subscription with the pubsub service.
    ///
    /// The returned stream will yield [messages](api::PubsubMessage) along with corresponding
    /// [`AcknowledgeTokens`](AcknowledgeToken), used to control message re-delivery.
    pub fn stream_subscription(
        &mut self,
        subscription: ProjectSubscriptionName,
        config: StreamSubscriptionConfig,
    ) -> StreamSubscription<S> {
        let sub_name: String = subscription.clone().into();
        let span = debug_span!("create_subscription", topic = sub_name);
        let _guard = span.enter();
        StreamSubscription::new(
            // As of the ack handler changes, the streaming implementation needs more than one
            // client. The obvious approach would be to clone the client as necessary. However
            // adding a Clone bound is a major semver change, and for downstream-simplification
            // reasons we'd like to avoid that.
            //
            // Fortunately, there already *was* a clone bound on this `stream_subscription`
            // function, which is the only public way to construct the stream. Also fortunately we
            // only need a static number of clients and not arbitrary clones, so we can clone here
            // and pass an array down there. This isn't *pretty*, but it works
            //
            // TODO(0.12.0) just add the clone bound
            [
                self.inner.clone(),
                self.inner.clone(),
                self.inner.clone(),
                self.inner.clone(),
            ],
            subscription.into(),
            config,
        )
    }
}

/// A project and subscription name combined in a format expected by API calls,
///
/// ```
/// use ya_gcp::pubsub::ProjectSubscriptionName;
///
/// assert_eq!(
///     String::from(ProjectSubscriptionName::new(
///         "my-project",
///         "my-subscription"
///     )),
///     "projects/my-project/subscriptions/my-subscription".to_string(),
/// );
/// ```
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ProjectSubscriptionName(String);

impl ProjectSubscriptionName {
    /// Create a new `ProjectSubscriptionName` from the given project and subscription names
    pub fn new(project_name: impl Display, subscription_name: impl Display) -> Self {
        Self(format!(
            "projects/{project}/subscriptions/{subscription}",
            project = project_name,
            subscription = subscription_name
        ))
    }
}

impl From<ProjectSubscriptionName> for String {
    fn from(from: ProjectSubscriptionName) -> String {
        from.0
    }
}

impl std::fmt::Display for ProjectSubscriptionName {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        std::fmt::Display::fmt(&self.0, f)
    }
}

/// A project and topic name combined in a format expected by API calls
///
/// ```
/// use ya_gcp::pubsub::ProjectTopicName;
///
/// assert_eq!(
///     String::from(ProjectTopicName::new("my-project", "my-topic")),
///     "projects/my-project/topics/my-topic".to_string(),
/// );
/// ```
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ProjectTopicName(String);

impl ProjectTopicName {
    /// Create a new `ProjectTopicName` from the given project and topic names
    pub fn new(project_name: impl Display, topic_name: impl Display) -> Self {
        Self(format!(
            "projects/{project}/topics/{topic}",
            project = project_name,
            topic = topic_name,
        ))
    }
}

impl From<ProjectTopicName> for String {
    fn from(from: ProjectTopicName) -> String {
        from.0
    }
}

impl std::fmt::Display for ProjectTopicName {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        std::fmt::Display::fmt(&self.0, f)
    }
}

/// The default [`RetryPredicate`] used for errors from PubSub operations
#[derive(Debug, Default, Clone)]
pub struct PubSubRetryCheck {
    _priv: (),
}

impl PubSubRetryCheck {
    /// Create a new instance with default settings
    pub fn new() -> Self {
        Self { _priv: () }
    }
}

impl RetryPredicate<Error> for PubSubRetryCheck {
    fn is_retriable(&self, error: &Error) -> bool {
        use tonic::Code;

        // this error code check is based on the ones used in the Java and Go pubsub client libs:
        // https://github.com/googleapis/java-pubsub/blob/d969e8925edc3401e6eb534699ce0351a5f0b20b/google-cloud-pubsub/src/main/java/com/google/cloud/pubsub/v1/StatusUtil.java#L33
        // https://github.com/googleapis/google-cloud-go/blob/ac9924157f35a00ff9d1e6ece9a7e0f12fc60226/pubsub/service.go#L51
        // Go doesn't retry on cancelled, but Java does; Java always retries Unknown, but Go only
        // does when the message implies "goaway". This takes a broad approach and retries on both
        //
        // This may need adjustment based on lower layers from the rust ecosystem, for example if
        // tonic interprets h2 errors and forwards as Internal/Unknown for particular cases. For
        // example, we could inspect the h2::Reason to discern NO_ERROR/GOAWAY from other Unknown
        // errors. For now, this is left as permissive for simplicity

        match error.code() {
            Code::DeadlineExceeded
            | Code::Internal
            | Code::Cancelled
            | Code::ResourceExhausted
            | Code::Aborted
            | Code::Unknown => true,
            Code::Unavailable => {
                let is_shutdown = error.message().contains("Server shutdownNow invoked");
                !is_shutdown
            }
            _ => false,
        }
    }
}
