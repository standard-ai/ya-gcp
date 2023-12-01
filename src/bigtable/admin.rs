//! An API for administering bigtable.

use super::api::bigtable::admin;
use crate::{
    builder,
    grpc::{Body, BoxBody, Bytes, DefaultGrpcImpl, GrpcService, StdError},
};

pub use admin::v2::Table;
use futures::Stream;

// TODO: https://cloud.google.com/bigtable/docs/reference/admin/rpc/google.bigtable.admin.v2#google.bigtable.admin.v2.BigtableTableAdmin
// lists lots of different scopes. What's the difference between them?
const BIGTABLE_ADMIN_SCOPE: &'static str = "https://www.googleapis.com/auth/bigtable.admin";

config_default! {
    /// Configuration for the bigtable admin client
    #[derive(Debug, Clone, Eq, PartialEq, Hash, serde::Deserialize)]
    pub struct BigtableTableAdminConfig {
        /// Endpoint to connect to bigtable over.
        @default("https://bigtableadmin.googleapis.com".into(), "BigtableTableAdminConfig::default_endpoint")
        pub endpoint: String,
    }
}

/// A client for connecting to bigtable. Created from the
/// [`build_bigtable_client`](crate::builder::ClientBuilder::build_bigtable_client)
/// function.
#[derive(Clone)]
pub struct BigtableTableAdminClient<S = DefaultGrpcImpl> {
    pub(crate) inner: admin::v2::bigtable_table_admin_client::BigtableTableAdminClient<S>,
    // A string of the form projects/{project}/instances/{instance}
    pub(crate) table_prefix: String,
}

pub use admin::v2::gc_rule::Rule;

impl Rule {
    /// Take the union of this rule with `other`.
    pub fn union(self, other: Rule) -> Rule {
        match self {
            Rule::Union(mut union) => {
                union.rules.push(other.into());
                Rule::Union(union)
            }
            r => Rule::Union(admin::v2::gc_rule::Union {
                rules: vec![r.into(), other.into()],
            }),
        }
    }

    /// Take the intersection of this rule with `other`.
    pub fn intersection(self, other: Rule) -> Rule {
        match self {
            Rule::Intersection(mut int) => {
                int.rules.push(other.into());
                Rule::Intersection(int)
            }
            r => Rule::Intersection(admin::v2::gc_rule::Intersection {
                rules: vec![r.into(), other.into()],
            }),
        }
    }
}

impl From<Rule> for admin::v2::GcRule {
    fn from(rule: Rule) -> admin::v2::GcRule {
        admin::v2::GcRule { rule: Some(rule) }
    }
}

impl<S> BigtableTableAdminClient<S> {
    /// Manually construct a new client.
    ///
    /// There are limited circumstances in which this is useful; consider instead using the builder
    /// function [crate::builder::ClientBuilder::build_bigtable_admin_client]
    pub fn from_raw_api(
        client: admin::v2::bigtable_table_admin_client::BigtableTableAdminClient<S>,
        project: &str,
        instance_name: &str,
    ) -> Self {
        BigtableTableAdminClient {
            inner: client,
            table_prefix: format!("projects/{}/instances/{}", project, instance_name),
        }
    }

    /// Access the underlying grpc api
    pub fn raw_api(&self) -> &admin::v2::bigtable_table_admin_client::BigtableTableAdminClient<S> {
        &self.inner
    }

    /// Mutably access the underlying grpc api
    pub fn raw_api_mut(
        &mut self,
    ) -> &mut admin::v2::bigtable_table_admin_client::BigtableTableAdminClient<S> {
        &mut self.inner
    }
}

impl<S> BigtableTableAdminClient<S>
where
    S: GrpcService<BoxBody>,
    S::Error: Into<StdError>,
    S::ResponseBody: Body<Data = Bytes> + Send + 'static,
    <S::ResponseBody as Body>::Error: Into<StdError> + Send,
{
    /// Create a new table.
    pub async fn create_table<Fams>(
        &mut self,
        table_name: impl Into<String>,
        column_families: Fams,
    ) -> Result<Table, tonic::Status>
    where
        Fams: IntoIterator<Item = (String, Rule)>,
    {
        let table_id = table_name.into();
        let table = Table {
            column_families: column_families
                .into_iter()
                .map(|(name, rule)| {
                    (
                        name,
                        admin::v2::ColumnFamily {
                            gc_rule: Some(rule.into()),
                        },
                    )
                })
                .collect(),
            ..Default::default()
        };
        let req = admin::v2::CreateTableRequest {
            parent: self.table_prefix.clone(),
            table_id,
            table: Some(table),
            ..Default::default()
        };

        let response = self.inner.create_table(req).await?;
        Ok(response.into_inner())
    }

    /// List all tables belonging to this project and instance.
    pub async fn list_tables(
        &mut self,
    ) -> Result<impl Stream<Item = Result<Table, tonic::Status>> + '_, tonic::Status> {
        Ok(async_stream::stream! {
            let mut page_token = String::new();
            loop {
                let req = admin::v2::ListTablesRequest {
                    parent: self.table_prefix.clone(),
                    page_token,
                    ..Default::default()
                };
                let response = self.inner.list_tables(req).await?.into_inner();

                for table in response.tables.into_iter() {
                    yield Ok(table);
                }

                if response.next_page_token.is_empty() {
                    break;
                } else {
                    page_token = response.next_page_token;
                    continue;
                }
            }
        })
    }
}

/// An error encountered when building Bigtable table admin clients
#[derive(Debug, thiserror::Error)]
#[error(transparent)]
pub struct BuildError(#[from] tonic::transport::Error);

impl builder::ClientBuilder {
    /// Create a client for administering bigtable tables.
    pub async fn build_bigtable_admin_client(
        &self,
        config: BigtableTableAdminConfig,
        project: &str,
        instance_name: &str,
    ) -> Result<BigtableTableAdminClient<DefaultGrpcImpl>, BuildError> {
        let scopes = vec![BIGTABLE_ADMIN_SCOPE.to_owned()];
        let endpoint = tonic::transport::Endpoint::new(config.endpoint)?;

        let connection = endpoint.connect().await?;

        let inner = admin::v2::bigtable_table_admin_client::BigtableTableAdminClient::new(
            DefaultGrpcImpl::new(connection, self.auth.clone(), scopes),
        );

        Ok(BigtableTableAdminClient::from_raw_api(
            inner,
            project,
            instance_name,
        ))
    }
}
