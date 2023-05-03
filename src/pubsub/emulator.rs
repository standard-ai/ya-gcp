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

use futures::{future::BoxFuture, FutureExt};

use crate::{
    builder::ClientBuilder,
    emulator::{self, EmulatorData},
    pubsub,
};
use tracing::debug;

type BoxError = Box<dyn std::error::Error + Send + Sync + 'static>;

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
    /// Create a new emulator instance with a default project name
    pub async fn new() -> Result<Self, BoxError> {
        let temp = tempdir::TempDir::new("pubsub_emulator")?;
        Ok(EmulatorClient {
            inner: emulator::EmulatorClient::new(data(&temp)).await?,
            _temp: temp,
        })
    }

    /// Create a new emulator instance with the given project name
    pub async fn with_project(project_name: impl Into<String>) -> Result<Self, BoxError> {
        let temp = tempdir::TempDir::new("pubsub_emulator")?;
        debug!(
            path = temp.as_ref().to_str(),
            "Created emulator data directory"
        );

        let project_name = project_name.into();

        Ok(EmulatorClient {
            inner: emulator::EmulatorClient::with_project(data(&temp), project_name).await?,
            _temp: temp,
        })
    }

    /// Create a new emulator instance with the given project name, which retries
    /// connection the specified number of times.
    pub async fn with_project_and_connect_retry_limit(
        project_name: impl Into<String>,
        connect_retry_limit: usize,
    ) -> Result<Self, BoxError> {
        let temp = tempdir::TempDir::new("pubsub_emulator")?;
        Ok(EmulatorClient {
            inner: emulator::EmulatorClient::with_project_and_connect_retry_limit(
                data(&temp),
                project_name,
                connect_retry_limit,
            )
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
