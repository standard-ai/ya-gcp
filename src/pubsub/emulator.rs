//! Testing infra to make use of the pubsub emulator.
//! <https://cloud.google.com/pubsub/docs/emulator>
//!
//! Follow installation directions from link above to set up your local development. Once setup,
//! you should be able to run the pubsub emulator driven tests.
//!
//! For a new test, create a new instance of [`EmulatorClient`]. Under the hood, this will find an
//! open port and start a new pubsub emulator server. The port is used to create subscription and
//! publish clients on the `EmulatorClient` instance.
//!
//! Cleanup:
//! When the test ends (in success or failure) the Drop trait implementation of [`EmulatorClient`]
//! ensures the system cleans up after itself. To verify the system cleaned up after itself run
//! `ps aux | grep pubsub`. If there are open pubsub servers, run `pkill -f pubsub` to remove them
//! all.

use std::{future::IntoFuture, marker::PhantomData};

use futures::{future::BoxFuture, FutureExt};

use crate::{
    builder::ClientBuilder,
    emulator::{self, EmulatorData, CLIENT_CONNECT_RETRY_DEFAULT},
    pubsub,
};
use tracing::debug;

type BoxError = Box<dyn std::error::Error + Send + Sync + 'static>;

mod builder_state {
    pub trait State {}

    pub enum NotReady {}
    impl State for NotReady {}

    pub enum Ready {}
    impl State for Ready {}
}

/// An async builder for constructing an emulator.
pub struct Emulator<S: builder_state::State> {
    project: Option<String>,
    connection_retry_limit: usize,
    _state: PhantomData<S>,
}

impl Emulator<builder_state::NotReady> {
    /// Returns a new async builder for constructing an emulator.
    pub fn new() -> Self {
        Self {
            project: None,
            connection_retry_limit: CLIENT_CONNECT_RETRY_DEFAULT,
            _state: PhantomData,
        }
    }

    /// The GCP project name the emulator should use.
    pub fn project(self, project: impl Into<String>) -> Emulator<builder_state::Ready> {
        Emulator {
            project: Some(project.into()),
            connection_retry_limit: CLIENT_CONNECT_RETRY_DEFAULT,
            _state: PhantomData,
        }
    }
}

impl<S: builder_state::State> Emulator<S> {
    /// How many times the emulator client should attempt to connect to the
    /// emulator before giving up. Retries occur every 100ms so, e.g., a value
    /// of `50` will result in a total retry time of 5s.
    pub fn connection_retry_limit(mut self, connection_retry_limit: usize) -> Self {
        self.connection_retry_limit = connection_retry_limit;
        self
    }
}

impl IntoFuture for Emulator<builder_state::Ready> {
    type Output = Result<EmulatorClient, BoxError>;
    type IntoFuture = BoxFuture<'static, Self::Output>;

    fn into_future(self) -> Self::IntoFuture {
        async move { EmulatorClient::new(self.project.unwrap(), self.connection_retry_limit).await }
            .boxed()
    }
}

/// Struct to hold a started PubSub emulator process. Process is closed when struct is dropped.
pub struct EmulatorClient {
    inner: crate::emulator::EmulatorClient,
    _temp: tempdir::TempDir,
}

fn data(tmp_dir: &tempdir::TempDir) -> EmulatorData {
    EmulatorData {
        gcloud_param: "pubsub",
        kill_pattern: "pubsub",
        availability_check: create_schema_client,
        extra_args: vec!["--data-dir".into(), tmp_dir.path().into()],
    }
}

impl EmulatorClient {
    /// Create a new emulator instance with the given project name, which retries
    /// connection the specified number of times.
    async fn new(
        project_name: impl Into<String>,
        connect_retry_limit: usize,
    ) -> Result<Self, BoxError> {
        let temp = tempdir::TempDir::new("pubsub_emulator")?;
        Ok(EmulatorClient {
            inner: emulator::EmulatorClient::new(data(&temp), project_name, connect_retry_limit)
                .await?,
            _temp: temp,
        })
    }

    /// Get the endpoint at which the emulator is listening for requests
    pub fn endpoint(&self) -> String {
        self.inner.endpoint()
    }

    /// Get the project name with which the emulator was initialized
    pub fn project(&self) -> &str {
        self.inner.project()
    }

    /// Get a client builder which is pre-configured to work with this emulator instance
    pub fn builder(&self) -> &ClientBuilder {
        self.inner.builder()
    }

    /// Create a new topic under this emualtor's given project name
    pub async fn create_topic(&self, topic_name: impl AsRef<str>) -> Result<(), BoxError> {
        let config = pubsub::PubSubConfig {
            endpoint: self.endpoint(),
            ..pubsub::PubSubConfig::default()
        };

        let mut publisher = self.builder().build_pubsub_publisher(config).await?;

        publisher
            .raw_api_mut()
            .create_topic(pubsub::api::Topic {
                name: pubsub::ProjectTopicName::new(self.project(), topic_name.as_ref()).into(),
                ..pubsub::api::Topic::default()
            })
            .await?;
        debug!(topic = topic_name.as_ref(), "Emulator created topic");
        Ok(())
    }
}

/// Creates the client to interact with PubSub schema service. Not currently supported by
/// emulator.
fn create_schema_client(port: &str) -> BoxFuture<Result<(), tonic::transport::Error>> {
    async move {
        pubsub::api::schema_service_client::SchemaServiceClient::connect(format!(
            "http://{}:{}",
            crate::emulator::HOST,
            port
        ))
        .await?;
        Ok(())
    }
    .boxed()
}
