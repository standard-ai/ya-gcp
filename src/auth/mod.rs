//! Utilities for authentication and authorization with GCP services.

#[cfg(feature = "grpc")]
pub mod grpc;

pub(crate) type Auth = yup_oauth2::authenticator::Authenticator<
    hyper_rustls::HttpsConnector<hyper::client::HttpConnector>,
>;

/// Add the given authorization token to the given HTTP request
///
/// Returns an error if the token cannot form a valid HTTP header value.
#[allow(unused)] // not used if all features are disabled
pub(crate) fn add_auth_token<Token, RequestBody>(
    request: &mut http::Request<RequestBody>,
    token: Token,
) -> Result<(), http::header::InvalidHeaderValue>
where
    Token: AsRef<str>,
{
    let header = format!("Bearer {}", token.as_ref());

    let auth_header_value = http::header::HeaderValue::from_str(&header)?;

    request
        .headers_mut()
        .insert(http::header::AUTHORIZATION, auth_header_value);

    Ok(())
}
