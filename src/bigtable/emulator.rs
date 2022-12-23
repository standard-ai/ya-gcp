//! Testing infra to make use of the bigtable emulator.
//! <https://cloud.google.com/bigtable/docs/emulator>
//!
//! Follow installation directions from link above to set up your local development. Once setup,
//! you should be able to run the pubsub emulator driven tests.

use futures::{future::BoxFuture, FutureExt};

use crate::{
    bigtable,
    builder::ClientBuilder,
    emulator::{self, EmulatorData},
};

type BoxError = Box<dyn std::error::Error + Send + Sync + 'static>;

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

const INSTANCE_ID: &str = "test-instance";

impl EmulatorClient {
    /// Create a new emulator instance with a default project name and instance name
    pub async fn new() -> Result<Self, BoxError> {
        Ok(EmulatorClient {
            inner: emulator::EmulatorClient::new(DATA).await?,
            instance: INSTANCE_ID.into(),
        })
    }

    /// Create a new emulator instance with the given project name and instance name
    pub async fn with_project_and_instance(
        project_name: impl Into<String>,
        instance_name: impl Into<String>,
    ) -> Result<Self, BoxError> {
        Ok(EmulatorClient {
            inner: emulator::EmulatorClient::with_project(DATA, project_name).await?,
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
