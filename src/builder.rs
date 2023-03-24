//! Items and functions used to create new service clients.
//!
//! See [`ClientBuilder`], which is used to instantiate the various GCP service clients.

use std::path::PathBuf;

use hyper::client::Client;

use crate::Auth;

const SERVICE_ACCOUNT_ENV_VAR: &str = "GOOGLE_APPLICATION_CREDENTIALS";

/// Configuration for loading service account credentials from file
#[derive(Debug, Clone, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
#[non_exhaustive]
pub enum ServiceAccountAuth {
    /// Specifies that the service account credentials should be read from the path stored in the
    /// environment variable `GOOGLE_APPLICATION_CREDENTIALS`
    EnvVar,

    /// Specifies that the service account credentials should be read from the given path
    Path(PathBuf),

    /// Use the Application Default Service Account, which is often attached to a specific
    /// compute instance via metadata
    ApplicationDefault,
}

impl Default for ServiceAccountAuth {
    fn default() -> Self {
        Self::EnvVar
    }
}

/// A marker to choose the mechanism by which authentication credentials should be loaded
#[derive(Debug, Clone, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
#[non_exhaustive]
pub enum AuthFlow {
    /// Load credentials for a service account
    ServiceAccount(ServiceAccountAuth),

    /// Load credentials from an authorized user secret, such as the one created when
    /// running `gcloud auth application-default login`
    UserAccount(PathBuf),

    /// Impersonate a service account from a user account
    ServiceAccountImpersonation {
        /// Path to the user credentials
        user: PathBuf,
        /// Email address of the service account
        email: String,
    },

    /// Skip authentication.
    ///
    /// Requests will not include any authorization header. This can be useful for tests
    NoAuth,
    // TODO consider support for InstalledFlow, DeviceFlow, etc
}

impl Default for AuthFlow {
    fn default() -> Self {
        AuthFlow::ServiceAccount(ServiceAccountAuth::default())
    }
}

config_default! {
    /// Configuration for creating a [`ClientBuilder`]
    #[derive(Debug, Clone, serde::Deserialize)]
    #[non_exhaustive]
    pub struct ClientBuilderConfig {
        /// How authentication credentials should be loaded
        @default(AuthFlow::ServiceAccount(ServiceAccountAuth::EnvVar), "ClientBuilderConfig::default_auth_flow")
        pub auth_flow: AuthFlow,
    }
}

/// The possible errors encountered when creating a [`ClientBuilder`]
#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
pub enum CreateBuilderError {
    /// An error in loading service account credentials from file
    #[error("failed to read service account key {}", _1.display())]
    ReadServiceAccountKey(#[source] std::io::Error, PathBuf),

    /// An error in loading user account credentials from file
    #[error("failed to read user account secrets {}", _1.display())]
    ReadUserSecrets(#[source] std::io::Error, PathBuf),

    /// An error in reading an environment variable
    #[error(
        "environment variable {SERVICE_ACCOUNT_ENV_VAR} isn't set. Consider either setting the \
         variable, or specifying the path with ServiceAccountAuth::Path(...)"
    )]
    CredentialsVarMissing,

    /// An error in initializing the authenticator
    #[error("failed to initialize authenticator")]
    Authenticator(#[source] std::io::Error),

    /// An error in initializing the HTTP connector
    #[error("failed to initialize HTTP connector")]
    Connector(#[source] Box<dyn std::error::Error + Send + Sync + 'static>),
}

// Set a default connector if a connector feature is enabled.
//
// This is default is then used as a default type for the generic parameter on types throughout
// this crate. Using this default is more ergonomic than specifying a generic type everywhere,
// which suits this crate's goal of ease-of-use
cfg_if::cfg_if! {
    // the order of the checked features has potentially user-facing consequences: "rustls" is
    // currently enabled in default features, so "openssl" should be checked in this if-chain
    // before it, so that `features = ["openssl"]` will do the less surprising thing (use openssl)
    // despite having rustls and openssl both enabled. This allows a user to forget
    // `default-features = false` with hopefully fewer unexpected consequences

    if #[cfg(feature="openssl")] {
        /// The default connector used for clients, based on the crate's enabled features
        pub type DefaultConnector =
            hyper_openssl::HttpsConnector<hyper::client::connect::HttpConnector>;

        impl ClientBuilder {
            /// Create a new client builder using the default HTTPS connector based on the crate's
            /// enabled features
            #[cfg_attr(docsrs, doc(cfg(any(feature="rustls", feature="openssl"))))]
            pub async fn new(config: ClientBuilderConfig) -> Result<Self, CreateBuilderError> {
                let connector = hyper_openssl::HttpsConnector::new()
                    .map_err(|e| CreateBuilderError::Connector(e.into()))?;

                Self::with_connector(config, connector).await
            }
        }
    }
    else if #[cfg(feature="rustls")] {
        /// The default connector used for clients, based on the crate's enabled features
        pub type DefaultConnector =
            hyper_rustls::HttpsConnector<hyper::client::connect::HttpConnector>;

        impl ClientBuilder {
            /// Create a new client builder using the default HTTPS connector based on the crate's
            /// enabled features
            #[cfg_attr(docsrs, doc(cfg(any(feature="rustls", feature="openssl"))))]
            pub async fn new(config: ClientBuilderConfig) -> Result<Self, CreateBuilderError> {
                let connector = hyper_rustls::HttpsConnector::with_native_roots();

                Self::with_connector(config, connector).await
            }
        }
    }
    else {
        // If no connector features are enabled, the default connector can be any type. The type
        // may show up in errors if a user forgets to specify the generic, however, so it's useful
        // to have the type's name document the user's issue

        #[doc(hidden)]
        pub struct NoConnectorFeaturesEnabled;

        /// The default connector used for clients, based on the crate's enabled features
        pub type DefaultConnector = NoConnectorFeaturesEnabled;
    }
}

/// A builder used to create all the clients for interacting with GCP services.
///
/// Note that the builder is not consumed when creating clients, and many clients can be built
/// using the same builder. This may allow some resource re-use across the clients
pub struct ClientBuilder<Connector = DefaultConnector> {
    // not all features use all the fields. Suppress the unused warning for simplicity
    #[allow(unused)]
    pub(crate) connector: Connector,
    #[allow(unused)]
    pub(crate) auth: Option<Auth<Connector>>,
    #[allow(unused)]
    pub(crate) client: Client<Connector>,
}

impl<C> ClientBuilder<C> {
    /// Create a new client builder using the given connector
    pub async fn with_connector(
        config: ClientBuilderConfig,
        connector: C,
    ) -> Result<Self, CreateBuilderError>
    where
        C: hyper::service::Service<http::Uri> + Clone + Send + Sync + 'static,
        C::Response: hyper::client::connect::Connection
            + tokio::io::AsyncRead
            + tokio::io::AsyncWrite
            + Send
            + Unpin
            + 'static,
        C::Future: Send + Unpin + 'static,
        C::Error: Into<Box<dyn std::error::Error + Send + Sync>>,
    {
        use AuthFlow::{NoAuth, ServiceAccount, ServiceAccountImpersonation, UserAccount};

        let client = hyper::client::Client::builder().build(connector.clone());

        let auth = match config.auth_flow {
            NoAuth => None,
            ServiceAccount(service_config) => Some(
                create_service_auth(
                    match service_config {
                        ServiceAccountAuth::Path(path) => Some(path.into_os_string()),
                        ServiceAccountAuth::EnvVar => Some(
                            std::env::var_os(SERVICE_ACCOUNT_ENV_VAR)
                                .ok_or(CreateBuilderError::CredentialsVarMissing)?,
                        ),
                        ServiceAccountAuth::ApplicationDefault => None,
                    },
                    client.clone(),
                )
                .await?,
            ),
            ServiceAccountImpersonation { user, email } => Some(
                create_service_impersonation_auth(user.into_os_string(), email, client.clone())
                    .await?,
            ),
            UserAccount(path) => {
                Some(create_user_auth(path.into_os_string(), client.clone()).await?)
            }
        };

        Ok(Self {
            connector,
            client,
            auth,
        })
    }

    /// Create a new client builder with the given connector and auth builder.
    pub async fn with_connector_and_auth_builder<F>(
        connector: C,
        auth_builder: impl FnOnce(Client<C>) -> F,
    ) -> Result<Self, CreateBuilderError>
    where
        C: crate::Connect + Clone + Send + Sync + 'static,
        F: futures::Future<Output = std::io::Result<Auth<C>>>,
    {
        let client = hyper::client::Client::builder().build(connector.clone());

        let auth = Some(
            auth_builder(client.clone())
                .await
                .map_err(CreateBuilderError::Authenticator)?,
        );

        Ok(Self {
            connector,
            client,
            auth,
        })
    }
}

/// Convenience method to create an Authorization for the oauth ServiceFlow.
async fn create_service_auth<C>(
    service_account_key_path: Option<impl AsRef<std::path::Path>>,
    client: Client<C>,
) -> Result<Auth<C>, CreateBuilderError>
where
    C: hyper::service::Service<http::Uri> + Clone + Send + Sync + 'static,
    C::Response: hyper::client::connect::Connection
        + tokio::io::AsyncRead
        + tokio::io::AsyncWrite
        + Send
        + Unpin
        + 'static,
    C::Future: Send + Unpin + 'static,
    C::Error: Into<Box<dyn std::error::Error + Send + Sync>>,
{
    match service_account_key_path {
        Some(service_account_key_path) => {
            let service_account_key =
                yup_oauth2::read_service_account_key(service_account_key_path.as_ref())
                    .await
                    .map_err(|e| {
                        CreateBuilderError::ReadServiceAccountKey(
                            e,
                            service_account_key_path.as_ref().to_owned(),
                        )
                    })?;

            yup_oauth2::ServiceAccountAuthenticator::builder(service_account_key)
                .hyper_client(client)
                .build()
                .await
                .map_err(CreateBuilderError::Authenticator)
        }
        None => match yup_oauth2::ApplicationDefaultCredentialsAuthenticator::with_client(
            yup_oauth2::ApplicationDefaultCredentialsFlowOpts::default(),
            client,
        )
        .await
        {
            yup_oauth2::authenticator::ApplicationDefaultCredentialsTypes::ServiceAccount(auth) => {
                auth.build()
                    .await
                    .map_err(CreateBuilderError::Authenticator)
            }
            yup_oauth2::authenticator::ApplicationDefaultCredentialsTypes::InstanceMetadata(
                auth,
            ) => auth
                .build()
                .await
                .map_err(CreateBuilderError::Authenticator),
        },
    }
}

async fn create_user_auth<C>(
    user_secrets_path: impl AsRef<std::path::Path>,
    client: Client<C>,
) -> Result<Auth<C>, CreateBuilderError>
where
    C: hyper::service::Service<http::Uri> + Clone + Send + Sync + 'static,
    C::Response: hyper::client::connect::Connection
        + tokio::io::AsyncRead
        + tokio::io::AsyncWrite
        + Send
        + Unpin
        + 'static,
    C::Future: Send + Unpin + 'static,
    C::Error: Into<Box<dyn std::error::Error + Send + Sync>>,
{
    let user_secret = yup_oauth2::read_authorized_user_secret(user_secrets_path.as_ref())
        .await
        .map_err(|e| {
            CreateBuilderError::ReadUserSecrets(e, user_secrets_path.as_ref().to_owned())
        })?;

    yup_oauth2::AuthorizedUserAuthenticator::with_client(user_secret, client)
        .build()
        .await
        .map_err(CreateBuilderError::Authenticator)
}

async fn create_service_impersonation_auth<C>(
    user_secrets_path: impl AsRef<std::path::Path>,
    email: String,
    client: Client<C>,
) -> Result<Auth<C>, CreateBuilderError>
where
    C: hyper::service::Service<http::Uri> + Clone + Send + Sync + 'static,
    C::Response: hyper::client::connect::Connection
        + tokio::io::AsyncRead
        + tokio::io::AsyncWrite
        + Send
        + Unpin
        + 'static,
    C::Future: Send + Unpin + 'static,
    C::Error: Into<Box<dyn std::error::Error + Send + Sync>>,
{
    let user_secret = yup_oauth2::read_authorized_user_secret(user_secrets_path.as_ref())
        .await
        .map_err(|e| {
            CreateBuilderError::ReadUserSecrets(e, user_secrets_path.as_ref().to_owned())
        })?;

    yup_oauth2::ServiceAccountImpersonationAuthenticator::with_client(user_secret, &email, client)
        .build()
        .await
        .map_err(CreateBuilderError::Authenticator)
}
