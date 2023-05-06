//! Testing infra to make use of GCP emulators.
//! <https://cloud.google.com/pubsub/docs/emulator>
//! <https://cloud.google.com/bigtable/docs/emulator>
//!
//! Follow installation directions from links above to set up your local development. Once setup,
//! you should be able to run the pubsub emulator driven tests.
//!
//! For a new test, create a new instance of [`EmulatorClient`]. Under the hood, this will find an
//! open port and start a new pubsub emulator server. The port is used to create subscription and
//! publish clients on the `EmulatorClient` instance.
//!
//! Cleanup:
//! When the test ends (in success or failure) the Drop trait implementation of [`EmulatorClient`]
//! ensures the system cleans up after itself. To verify the system cleaned up after itself run
//! `ps aux | grep pubsub`. If there are open pubsub/bigtable servers, run `pkill -f $service`
//! (where `$service` is one of either `pubsub` or `bigtable`) to remove them all.

use std::{ffi::OsString, ops::Range};

use crate::builder::{AuthFlow, ClientBuilder, ClientBuilderConfig};
use futures::future::BoxFuture;
use rand::{self, Rng};
use tracing::debug;

type BoxError = Box<dyn std::error::Error + Send + Sync + 'static>;

const PORT_RANGE: Range<usize> = 8000..12000;
pub(crate) const HOST: &str = "localhost";
const CLI_RETRY: usize = 100;
pub(crate) const CLIENT_CONNECT_RETRY_DEFAULT: usize = 50;

#[derive(Clone)]
pub(crate) struct EmulatorData {
    // The command `gcloud beta emulators <what goes here?> start` starts the emulator.
    pub(crate) gcloud_param: &'static str,
    // Extra parameters to be passed to `gcloud beta emulators <something> start`.
    pub(crate) extra_args: Vec<OsString>,
    // To shut down extra emulator processes, search for this string.
    pub(crate) kill_pattern: &'static str,
    // A function returning `Ok` if the emulator has finished starting up.
    pub(crate) availability_check: fn(&str) -> BoxFuture<Result<(), tonic::transport::Error>>,
}

/// Struct to hold a started PubSub emulator process. Process is closed when struct is dropped.
pub(crate) struct EmulatorClient {
    // Child process running the pubsub emulator. Killed on drop, `wait`ed by tokio in the
    // background afterward.
    _child: tokio::process::Child,

    // The port where the emulator is running. e.g. 8050
    port: String,

    builder: ClientBuilder,

    project_name: String,

    data: EmulatorData,
}

impl EmulatorClient {
    /// Create a new emulator instance wiht the given `EmulatorData`, project name
    /// and limit to the number of connection retries.
    pub(crate) async fn new(
        data: EmulatorData,
        project_name: impl Into<String>,
        connection_retry_limit: usize,
    ) -> Result<Self, BoxError> {
        let project_name = project_name.into();

        let (child, port) =
            start_emulator(data.gcloud_param, &project_name, &data.extra_args).await?;
        debug!("Started emulator");

        // Give the server some time (5s) to come up.
        let mut err: Option<tonic::transport::Error> = None;
        let check = data.availability_check;
        for _ in 0..connection_retry_limit {
            // If we are able to create a schema client, then the server is up.
            match check(&port).await {
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
            project_name,
            data,
        })
    }

    /// Get the endpoint at which the emulator is listening for requests
    pub(crate) fn endpoint(&self) -> String {
        format!("http://{}:{}/v1", HOST, self.port)
    }

    /// Get the project name with which the emulator was initialized
    pub(crate) fn project(&self) -> &str {
        &self.project_name
    }

    /// Get a client builder which is pre-configured to work with this emulator instance
    pub(crate) fn builder(&self) -> &ClientBuilder {
        &self.builder
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
                .arg(format!(
                    ".*{}.*--port={}.*",
                    self.data.kill_pattern, self.port
                ))
                .output(),
        );
    }
}

/// Attempt to start the PubSub emulator. Sometimes theres a port collision and this might fail
/// so attempt it multiple times.
async fn start_emulator(
    emulator_name: &str,
    project_name: &str,
    extra_args: &[OsString],
) -> Result<(tokio::process::Child, String), std::io::Error> {
    let mut err: Option<_> = None;

    for _ in 0..CLI_RETRY {
        let port = format!("{}", rand::thread_rng().gen_range(PORT_RANGE));
        match start_emulator_once(emulator_name, &port, project_name, extra_args) {
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
    emulator_name: &str,
    port: &str,
    project_name: &str,
    extra_args: &[OsString],
) -> Result<tokio::process::Child, std::io::Error> {
    tokio::process::Command::new("gcloud")
        .arg("beta")
        .arg("emulators")
        .arg(emulator_name)
        .arg("start")
        .arg("--project")
        .arg(project_name)
        .arg("--host-port")
        .arg(format!("{}:{}", HOST, port))
        .args(extra_args)
        .arg("--verbosity")
        .arg("debug")
        .kill_on_drop(true)
        .spawn()
}
