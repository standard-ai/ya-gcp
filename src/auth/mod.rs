//! Utilities for authentication and authorization with GCP services.
use std::convert::TryInto;

#[cfg(feature = "grpc")]
pub mod grpc;

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
    request.headers_mut().insert(
        http::header::AUTHORIZATION,
        format!("Bearer {}", token.as_ref()).try_into()?,
    );

    Ok(())
}
