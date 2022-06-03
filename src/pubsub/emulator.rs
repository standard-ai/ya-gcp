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

use std::ops::Range;

use crate::{
    builder::{AuthFlow, ClientBuilder, ClientBuilderConfig},
    pubsub,
};
use rand::{self, Rng};
use std::{convert::TryInto, path::Path};

type BoxError = Box<dyn std::error::Error + Send + Sync + 'static>;

const PUBSUB_PROJECT_ID: &str = "test-project";
const PORT_RANGE: Range<usize> = 8000..12000;
const HOST: &str = "localhost";
const PUBSUB_CLI_RETRY: usize = 100;
const CLIENT_CONNECT_RETRY: usize = 50;

/// Struct to hold a started PubSub emulator process. Process is closed when struct is dropped.
pub struct EmulatorClient {
    // Child process running the pubsub emulator. Killed on drop, `wait`ed by tokio in the
    // background afterward.
    _child: tokio::process::Child,

    // The port where the emulator is running. e.g. 8050
    port: String,

    _temp: tempdir::TempDir,

    builder: ClientBuilder,

    project_name: String,
}

impl EmulatorClient {
    /// Create a new emulator instance with a default project name
    pub async fn new() -> Result<Self, BoxError> {
        Self::with_project(PUBSUB_PROJECT_ID).await
    }

    /// Create a new emulator instance with the given project name
    pub async fn with_project(project_name: impl Into<String>) -> Result<Self, BoxError> {
        // Create a tmp dir where pubsub can store its data. Removed when EmulatorClient drops
        let temp = tempdir::TempDir::new("pubsub_emulator")?;

        let project_name = project_name.into();

        let (child, port) = start_emulator(temp.path(), &project_name).await?;

        // Give the server some time (5s) to come up.
        let mut err: Option<tonic::transport::Error> = None;
        for _ in 0..CLIENT_CONNECT_RETRY {
            // If we are able to create a schema client, then the server is up.
            match create_schema_client(&port).await {
                Err(e) => {
                    err = Some(e);
                    tokio::time::sleep(std::time::Duration::from_millis(100)).await;
                }
                Ok(_) => {
                    err = None;
                    break;
                }
            }
        }

        if let Some(err) = err {
            return Err(err.into());
        }

        let config = ClientBuilderConfig {
            auth_flow: AuthFlow::NoAuth,
        };
        let builder = ClientBuilder::new(config).await?;

        Ok(Self {
            _child: child,
            port,
            builder,
            _temp: temp,
            project_name,
        })
    }

    /// Get the endpoint at which the emulator is listening for requests
    pub fn endpoint(&self) -> http::Uri {
        format!("http://{}:{}", HOST, self.port)
            .try_into()
            .expect("should form valid URI")
    }

    /// Get the project name with which the emulator was initialized
    pub fn project(&self) -> &str {
        &self.project_name
    }

    /// Get a client builder which is pre-configured to work with this emulator instance
    pub fn builder(&self) -> &ClientBuilder {
        &self.builder
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
        Ok(())
    }
}

/// Hacked callback to ensure the emulator process cleans up after itself.
impl Drop for EmulatorClient {
    fn drop(&mut self) {
        // Search by name for other pubsub processes with --port=PORT in the cmd name and kill
        // them. https://googlecloudplatform.uservoice.com/forums/302631-cloud-pub-sub/suggestions/42574147-java-process-spawned-when-pub-sub-emulator-runs-is
        tokio::spawn(
            tokio::process::Command::new("pkill")
                .arg("-f")
                .arg(format!(".*pubsub.*--port={}.*", self.port))
                .output(),
        );
    }
}

/// Attempt to start the PubSub emulator. Sometimes theres a port collision and this might fail
/// so attempt it multiple times.
async fn start_emulator(
    tmp_dir: &Path,
    project_name: &str,
) -> Result<(tokio::process::Child, String), std::io::Error> {
    let mut err: Option<_> = None;
    let mut rng = rand::thread_rng();

    for _ in 0..PUBSUB_CLI_RETRY {
        let port = format!("{}", rng.gen_range(PORT_RANGE));
        match start_emulator_once(&port, tmp_dir, project_name) {
            Ok(child) => return Ok((child, port)),
            Err(e) => {
                err = Some(e);
                tokio::time::sleep(std::time::Duration::from_millis(100)).await;
            }
        }
    }
    Err(err.unwrap())
}

/// Attempt to start the PubSub emulator. Sometimes theres a port collision and this might fail.
fn start_emulator_once(
    port: &str,
    tmp_dir: &Path,
    project_name: &str,
) -> Result<tokio::process::Child, std::io::Error> {
    tokio::process::Command::new("gcloud")
        .arg("beta")
        .arg("emulators")
        .arg("pubsub")
        .arg("start")
        .arg("--project")
        .arg(project_name)
        .arg("--host-port")
        .arg(format!("{}:{}", HOST, port))
        .arg("--data-dir")
        .arg(tmp_dir)
        .arg("--verbosity")
        .arg("debug")
        .kill_on_drop(true)
        .spawn()
}

/// Creates the client to interact with PubSub schema service. Not currently supported by
/// emulator.
async fn create_schema_client(
    port: &str,
) -> Result<
    pubsub::api::schema_service_client::SchemaServiceClient<
        impl tonic::client::GrpcService<
            tonic::body::BoxBody,
            Error = tonic::transport::Error,
            ResponseBody = tonic::transport::Body,
        >,
    >,
    tonic::transport::Error,
> {
    pubsub::api::schema_service_client::SchemaServiceClient::connect(format!(
        "http://{}:{}",
        HOST, port
    ))
    .await
}
