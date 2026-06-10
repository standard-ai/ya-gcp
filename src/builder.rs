//! Items and functions used to create new service clients.
//!
//! See [`ClientBuilder`], which is used to instantiate the various GCP service clients.

use crate::auth::Auth;
use std::path::PathBuf;

#[cfg(any(feature = "rustls-native-certs", feature = "webpki-roots"))]
use std::path::Path;

const SERVICE_ACCOUNT_ENV_VAR: &str = "GOOGLE_APPLICATION_CREDENTIALS";

/// Configuration for loading service account credentials from file
#[derive(Debug, Clone, Default, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
#[non_exhaustive]
pub enum ServiceAccountAuth {
    /// Specifies that the service account credentials should be read from the path stored in the
    /// environment variable `GOOGLE_APPLICATION_CREDENTIALS`
    #[default]
    EnvVar,

    /// Specifies that the service account credentials should be read from the given path
    Path(PathBuf),

    /// Use the Application Default Service Account, which is often attached to a specific
    /// compute instance via metadata
    ApplicationDefault,
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
    /// An error in loading workload identity federation / external-account credentials from file
    #[error("failed to read external account credentials from {}", _1.display())]
    ReadExternalAccountCredential(#[source] std::io::Error, PathBuf),

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
    /// Create a new client builder.
    ///
    /// HTTP clients for GCP API calls use [`https_connector`]; yup-oauth2 uses its own default
    /// HTTPS client for token exchange.
    #[cfg(any(feature = "rustls-native-certs", feature = "webpki-roots"))]
    pub async fn new(config: ClientBuilderConfig) -> Result<Self, CreateBuilderError> {
        use AuthFlow::{NoAuth, ServiceAccount, ServiceAccountImpersonation, UserAccount};

        let auth = match config.auth_flow {
            NoAuth => None,
            ServiceAccount(service_config) => Some(
                create_service_auth(match service_config {
                    ServiceAccountAuth::Path(path) => Some(path.into_os_string()),
                    ServiceAccountAuth::EnvVar => Some(
                        std::env::var_os(SERVICE_ACCOUNT_ENV_VAR)
                            .ok_or(CreateBuilderError::CredentialsVarMissing)?,
                    ),
                    ServiceAccountAuth::ApplicationDefault => None,
                })
                .await?,
            ),
            ServiceAccountImpersonation { user, email } => {
                Some(create_service_impersonation_auth(user.into_os_string(), email).await?)
            }
            UserAccount(path) => Some(create_user_auth(path.into_os_string()).await?),
        };

        Ok(Self { auth })
    }
}

#[cfg(any(feature = "rustls-native-certs", feature = "webpki-roots"))]
fn is_external_account_json(contents: &str) -> bool {
    serde_json::from_str::<serde_json::Value>(contents)
        .ok()
        .and_then(|v| v.get("type")?.as_str().map(|t| t == "external_account"))
        .unwrap_or(false)
}

#[cfg(any(feature = "rustls-native-certs", feature = "webpki-roots"))]
fn log_external_account_credentials(path: &Path) {
    tracing::info!(
        auth_build = crate::AUTH_BUILD_ID,
        path = %path.display(),
        "ya-gcp: using external_account credentials"
    );
}

#[cfg(any(feature = "rustls-native-certs", feature = "webpki-roots"))]
async fn build_external_account_auth(path: &Path) -> Result<Auth, CreateBuilderError> {
    log_external_account_credentials(path);

    let secret = yup_oauth2::read_external_account_secret(path)
        .await
        .map_err(|e| CreateBuilderError::ReadExternalAccountCredential(e, path.to_owned()))?;

    yup_oauth2::ExternalAccountAuthenticator::builder(secret)
        .build()
        .await
        .map_err(CreateBuilderError::Authenticator)
}

/// Load WIF / external-account credentials from `GOOGLE_APPLICATION_CREDENTIALS` when set.
#[cfg(any(feature = "rustls-native-certs", feature = "webpki-roots"))]
async fn external_account_auth_from_gac_env() -> Result<Option<Auth>, CreateBuilderError> {
    let Ok(path_buf) = std::env::var(SERVICE_ACCOUNT_ENV_VAR).map(PathBuf::from) else {
        return Ok(None);
    };

    if !tokio::fs::try_exists(&path_buf).await.unwrap_or(false) {
        return Ok(None);
    }

    let Ok(contents) = tokio::fs::read_to_string(&path_buf).await else {
        return Ok(None);
    };

    if !is_external_account_json(&contents) {
        return Ok(None);
    }

    build_external_account_auth(&path_buf).await.map(Some)
}

/// Convenience method to create an Authorization for the oauth ServiceFlow.
#[cfg(any(feature = "rustls-native-certs", feature = "webpki-roots"))]
async fn create_service_auth(
    service_account_key_path: Option<impl AsRef<std::path::Path>>,
) -> Result<Auth, CreateBuilderError> {
    match service_account_key_path.as_ref().map(|p| p.as_ref()) {
        Some(path) => {
            let contents = tokio::fs::read_to_string(path)
                .await
                .map_err(|e| CreateBuilderError::ReadServiceAccountKey(e, path.to_owned()))?;

            if is_external_account_json(&contents) {
                return build_external_account_auth(path).await;
            }

            let service_account_key = yup_oauth2::read_service_account_key(path)
                .await
                .map_err(|e| CreateBuilderError::ReadServiceAccountKey(e, path.to_owned()))?;

            yup_oauth2::ServiceAccountAuthenticator::builder(service_account_key)
                .build()
                .await
                .map_err(CreateBuilderError::Authenticator)
        }
        None => {
            if let Some(auth) = external_account_auth_from_gac_env().await? {
                return Ok(auth);
            }

            match yup_oauth2::ApplicationDefaultCredentialsAuthenticator::builder(
                yup_oauth2::ApplicationDefaultCredentialsFlowOpts::default(),
            )
            .await
            {
                yup_oauth2::authenticator::ApplicationDefaultCredentialsTypes::ServiceAccount(
                    auth,
                ) => auth
                    .build()
                    .await
                    .map_err(CreateBuilderError::Authenticator),
                yup_oauth2::authenticator::ApplicationDefaultCredentialsTypes::InstanceMetadata(
                    auth,
                ) => auth
                    .build()
                    .await
                    .map_err(CreateBuilderError::Authenticator),
            }
        }
    }
}

#[cfg(any(feature = "rustls-native-certs", feature = "webpki-roots"))]
async fn create_user_auth(
    user_secrets_path: impl AsRef<std::path::Path>,
) -> Result<Auth, CreateBuilderError> {
    let user_secret = yup_oauth2::read_authorized_user_secret(user_secrets_path.as_ref())
        .await
        .map_err(|e| {
            CreateBuilderError::ReadUserSecrets(e, user_secrets_path.as_ref().to_owned())
        })?;

    yup_oauth2::AuthorizedUserAuthenticator::builder(user_secret)
        .build()
        .await
        .map_err(CreateBuilderError::Authenticator)
}

#[cfg(any(feature = "rustls-native-certs", feature = "webpki-roots"))]
async fn create_service_impersonation_auth(
    user_secrets_path: impl AsRef<std::path::Path>,
    email: String,
) -> Result<Auth, CreateBuilderError> {
    let user_secret = yup_oauth2::read_authorized_user_secret(user_secrets_path.as_ref())
        .await
        .map_err(|e| {
            CreateBuilderError::ReadUserSecrets(e, user_secrets_path.as_ref().to_owned())
        })?;

    yup_oauth2::ServiceAccountImpersonationAuthenticator::builder(user_secret, &email)
        .build()
        .await
        .map_err(CreateBuilderError::Authenticator)
}
