//! Items and functions used to create new service clients.
//!
//! See [`ClientBuilder`], which is used to instantiate the various GCP service clients.

use crate::auth::Auth;
use std::path::PathBuf;

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

type Client = hyper::client::Client<hyper_rustls::HttpsConnector<hyper::client::HttpConnector>>;

#[allow(unused)] // only used by some feature combinations
pub(crate) fn https_connector() -> hyper_rustls::HttpsConnector<hyper::client::HttpConnector> {
    #[allow(unused_mut)]
    let mut roots = rustls::RootCertStore::empty();

    #[cfg(feature = "rustls-native-certs")]
    roots.add_parsable_certificates(
        &rustls_native_certs::load_native_certs().expect("could not load native certs"),
    );

    #[cfg(feature = "webpki-roots")]
    roots.add_trust_anchors(webpki_roots::TLS_SERVER_ROOTS.iter().map(|ta| {
        rustls::OwnedTrustAnchor::from_subject_spki_name_constraints(
            ta.subject,
            ta.spki,
            ta.name_constraints,
        )
    }));

    let tls_config = rustls::client::ClientConfig::builder()
        .with_safe_defaults()
        .with_root_certificates(roots)
        .with_no_client_auth();

    hyper_rustls::HttpsConnectorBuilder::new()
        .with_tls_config(tls_config)
        .https_or_http()
        .enable_all_versions()
        .build()
}

/// A builder used to create all the clients for interacting with GCP services.
///
/// Note that the builder is not consumed when creating clients, and many clients can be built
/// using the same builder. This may allow some resource re-use across the clients
pub struct ClientBuilder {
    #[allow(unused)]
    pub(crate) auth: Option<Auth>,
}

impl ClientBuilder {
    /// Create a new client builder using default HTTPS settings
    #[cfg(any(feature = "rustls-native-certs", feature = "webpki-roots"))]
    pub async fn new(config: ClientBuilderConfig) -> Result<Self, CreateBuilderError> {
        Self::with_auth_connector(config, https_connector).await
    }

    /// Create a new client builder using the given connector for authentication requests
    pub async fn with_auth_connector(
        config: ClientBuilderConfig,
        connector_fn: impl FnOnce() -> hyper_rustls::HttpsConnector<hyper::client::HttpConnector>,
    ) -> Result<Self, CreateBuilderError> {
        use AuthFlow::{NoAuth, ServiceAccount, ServiceAccountImpersonation, UserAccount};
        let make_client = move || hyper::client::Client::builder().build(connector_fn());

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
                    make_client(),
                )
                .await?,
            ),
            ServiceAccountImpersonation { user, email } => Some(
                create_service_impersonation_auth(user.into_os_string(), email, make_client())
                    .await?,
            ),
            UserAccount(path) => {
                Some(create_user_auth(path.into_os_string(), make_client()).await?)
            }
        };

        Ok(Self { auth })
    }
}

/// Convenience method to create an Authorization for the oauth ServiceFlow.
async fn create_service_auth(
    service_account_key_path: Option<impl AsRef<std::path::Path>>,
    client: Client,
) -> Result<Auth, CreateBuilderError> {
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

async fn create_user_auth(
    user_secrets_path: impl AsRef<std::path::Path>,
    client: Client,
) -> Result<Auth, CreateBuilderError> {
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

async fn create_service_impersonation_auth(
    user_secrets_path: impl AsRef<std::path::Path>,
    email: String,
    client: Client,
) -> Result<Auth, CreateBuilderError> {
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
