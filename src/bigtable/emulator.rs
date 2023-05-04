//! Testing infra to make use of the bigtable emulator.
//! <https://cloud.google.com/bigtable/docs/emulator>
//!
//! Follow installation directions from link above to set up your local development. Once setup,
//! you should be able to run the pubsub emulator driven tests.

use std::future::IntoFuture;

use futures::{future::BoxFuture, FutureExt};

use crate::{
    bigtable,
    builder::ClientBuilder,
    emulator::{self, EmulatorData, CLIENT_CONNECT_RETRY_DEFAULT, PROJECT_ID_DEFAULT},
};

type BoxError = Box<dyn std::error::Error + Send + Sync + 'static>;

/// An async builder for constructing an emulator.
pub struct Emulator {
    project: String,
    instance: String,
    connection_retry_limit: usize,
}

impl Emulator {
    /// Returns a new async builder for constructing an emulator
    pub fn new() -> Self {
        Self {
            project: PROJECT_ID_DEFAULT.to_owned(),
            instance: INSTANCE_ID_DEFAULT.to_owned(),
            connection_retry_limit: CLIENT_CONNECT_RETRY_DEFAULT,
        }
    }

    /// The GCP project name the emulator should use.
    pub fn project(mut self, project: impl Into<String>) -> Self {
        self.project = project.into();
        self
    }

    /// The Bigtable instance name the emulator should use.
    pub fn instance(mut self, instance: impl Into<String>) -> Self {
        self.instance = instance.into();
        self
    }

    /// How many times the emulator client should attempt to connect to the
    /// emulator before giving up. Retries occur every 100ms so, e.g., a value
    /// of `50` will result in a total retry time of 5s.
    pub fn connection_retry_limit(mut self, connection_retry_limit: usize) -> Self {
        self.connection_retry_limit = connection_retry_limit;
        self
    }
}

impl IntoFuture for Emulator {
    type Output = Result<EmulatorClient, BoxError>;
    type IntoFuture = BoxFuture<'static, Self::Output>;

    fn into_future(self) -> Self::IntoFuture {
        async move {
            EmulatorClient::new(self.project, self.instance, self.connection_retry_limit).await
        }
        .boxed()
    }
}

/// Struct to hold a started PubSub emulator process. Process is closed when struct is dropped.
pub struct EmulatorClient {
    inner: crate::emulator::EmulatorClient,
    instance: String,
}

const DATA: EmulatorData = EmulatorData {
    gcloud_param: "bigtable",
    kill_pattern: "bigtable",
    availability_check: create_bigtable_client,
    extra_args: Vec::new(),
};

const INSTANCE_ID_DEFAULT: &str = "test-instance";

impl EmulatorClient {
    /// Create a new emulator instance with the given project name, instance name,
    /// which retries connection the specified number of times.
    async fn new(
        project_name: impl Into<String>,
        instance_name: impl Into<String>,
        connect_retry_limit: usize,
    ) -> Result<Self, BoxError> {
        Ok(EmulatorClient {
            inner: emulator::EmulatorClient::new(DATA, project_name, connect_retry_limit).await?,
            instance: instance_name.into(),
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

    /// Get the instance name with which the emulator was initialized
    pub fn instance(&self) -> &str {
        &self.instance
    }

    /// Get a client builder which is pre-configured to work with this emulator instance
    pub fn builder(&self) -> &ClientBuilder {
        self.inner.builder()
    }

    /// Create a new table under this emulator's given project name and instance name.
    ///
    /// The column families will be created wth effectively no garbage collection.
    pub async fn create_table(
        &self,
        table_name: &str,
        column_families: impl IntoIterator<Item = impl Into<String>>,
    ) -> Result<(), BoxError> {
        let config = bigtable::admin::BigtableTableAdminConfig {
            endpoint: self.endpoint(),
            ..bigtable::admin::BigtableTableAdminConfig::default()
        };

        let mut admin = self
            .builder()
            .build_bigtable_admin_client(config, &self.project(), &self.instance)
            .await?;

        let column_families = column_families
            .into_iter()
            .map(|name| (name.into(), bigtable::admin::Rule::MaxNumVersions(i32::MAX)));
        admin.create_table(table_name, column_families).await?;

        Ok(())
    }
}

fn create_bigtable_client(port: &str) -> BoxFuture<Result<(), tonic::transport::Error>> {
    async move {
        bigtable::api::bigtable::v2::bigtable_client::BigtableClient::connect(format!(
            "http://{}:{}",
            crate::emulator::HOST,
            port
        ))
        .await?;
        Ok(())
    }
    .boxed()
}
