//! An API for administering bigtable.

use crate::{
    auth::grpc::{self, AuthGrpcService},
    builder,
};

use super::api::bigtable::admin;

pub use admin::v2::Table;
use futures::Stream;

// TODO: https://cloud.google.com/bigtable/docs/reference/admin/rpc/google.bigtable.admin.v2#google.bigtable.admin.v2.BigtableTableAdmin
// lists lots of different scopes. What's the difference between them?
const BIGTABLE_ADMIN_SCOPE: &'static str = "https://www.googleapis.com/auth/bigtable.admin";

config_default! {
    /// Configuration for the bigtable admin client
    #[derive(Debug, Clone, Eq, PartialEq, Hash, serde::Deserialize)]
    #[non_exhaustive]
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
pub struct BigtableTableAdminClient<C = crate::DefaultConnector> {
    pub(crate) inner: admin::v2::bigtable_table_admin_client::BigtableTableAdminClient<
        AuthGrpcService<tonic::transport::Channel, C>,
    >,
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

impl<C> BigtableTableAdminClient<C>
where
    C: tower::Service<http::Uri> + Clone + Send + Sync + 'static,
    C::Response: hyper::client::connect::Connection
        + tokio::io::AsyncRead
        + tokio::io::AsyncWrite
        + Send
        + Unpin
        + 'static,
    C::Future: Send + Unpin + 'static,
    C::Error: Into<Box<dyn std::error::Error + Send + Sync>>,
{
    /// Create a new table.
    pub async fn create_table<Fams>(
        &mut self,
        table_name: &str,
        column_families: Fams,
    ) -> Result<Table, tonic::Status>
    where
        Fams: IntoIterator<Item = (String, Rule)>,
    {
        let table_id = table_name.to_owned();
        let table = Table {
            name: format!("{}/tables/{}", self.table_prefix, table_name),
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

    /// List all tables.
    pub async fn list_tables(
        &mut self,
    ) -> Result<impl Stream<Item = Result<Table, tonic::Status>>, tonic::Status> {
        let req = admin::v2::ListTablesRequest {
            parent: self.table_prefix.clone(),
            ..Default::default()
        };
        let response = self.inner.list_tables(req).await?;
        // TODO: if the response was paged, make a follow-up request.
        Ok(futures::stream::iter(
            response.into_inner().tables.into_iter().map(|x| Ok(x)),
        ))
    }
}

/// An error encountered when building Bigtable table admin clients
#[derive(Debug, thiserror::Error)]
#[error(transparent)]
pub struct BuildError(#[from] tonic::transport::Error);

impl<C> builder::ClientBuilder<C>
where
    C: tower::Service<http::Uri> + Clone + Send + Sync + 'static,
    C::Response: hyper::client::connect::Connection
        + tokio::io::AsyncRead
        + tokio::io::AsyncWrite
        + Send
        + Unpin
        + 'static,
    C::Future: Send + Unpin + 'static,
    C::Error: Into<Box<dyn std::error::Error + Send + Sync>>,
    Box<dyn std::error::Error + Send + Sync + 'static>: From<C::Error>,
{
    /// Create a client for administering bigtable tables.
    pub async fn build_bigtable_admin_client(
        &self,
        config: BigtableTableAdminConfig,
        project: &str,
        instance_name: &str,
    ) -> Result<BigtableTableAdminClient<C>, BuildError> {
        let scopes = vec![BIGTABLE_ADMIN_SCOPE.to_owned()];
        let endpoint = tonic::transport::Endpoint::new(config.endpoint)?;

        let connection = endpoint
            .connect_with_connector(self.connector.clone())
            .await?;
        let table_prefix = format!("projects/{}/instances/{}", project, instance_name);

        let inner = admin::v2::bigtable_table_admin_client::BigtableTableAdminClient::new(
            grpc::AuthGrpcService::new(connection, self.auth.clone(), scopes),
        );

        Ok(BigtableTableAdminClient {
            inner,
            table_prefix,
        })
    }
}
