//! Authorization support for gRPC requests

use futures::future::BoxFuture;
use std::{
    future::Future,
    sync::Arc,
    task::{Context, Poll},
};
use tonic::client::GrpcService;

/// A source of authorization tokens for RPC requests
pub trait TokenSource {
    /// The authorization token type
    type Token: AsRef<str>;
    /// The future returned by [`get_token`](TokenSource::get_token)
    type Fut: Future<Output = Result<Self::Token, Self::Error>> + Send + 'static;
    /// Errors that may occur while getting a token
    type Error: std::error::Error + Send + Sync + 'static;

    /// Get a valid authorization token, either reusing a cached one or fetching a new one from the
    /// authorization source
    fn get_token(&self) -> Self::Fut;
}

// blanket impl TokenSource for closures as a convenience
impl<F, Token, TokenFut, TokenErr> TokenSource for F
where
    F: Fn() -> TokenFut,
    TokenFut: Future<Output = Result<Token, TokenErr>> + Send + 'static,
    TokenErr: std::error::Error + Send + Sync + 'static,
    Token: AsRef<str>,
{
    type Error = TokenErr;
    type Fut = TokenFut;
    type Token = Token;

    fn get_token(&self) -> TokenFut {
        (self)()
    }
}

/// A [`TokenSource`](TokenSource) based on OAuth
#[derive(Clone)]
pub struct OAuthTokenSource<C = crate::builder::DefaultConnector> {
    oauth: crate::Auth<C>,
    scopes: Arc<[String]>,
}

impl<C> std::fmt::Debug for OAuthTokenSource<C> {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        f.debug_struct("OAuthTokenSource")
            .field("scopes", &self.scopes)
            .field("oauth", &"...")
            .finish()
    }
}

impl<C> TokenSource for OAuthTokenSource<C>
where
    C: hyper::client::connect::Connect + Clone + Send + Sync + 'static,
{
    type Error = yup_oauth2::Error;
    // TODO(type_alias_impl_trait) don't box
    type Fut = BoxFuture<'static, Result<Self::Token, Self::Error>>;
    type Token = yup_oauth2::AccessToken;

    fn get_token(&self) -> Self::Fut {
        // unfortunately many futures in tonic must be 'static, so they can't borrow and
        // must own all their contents. Hence these clones
        let oauth = self.oauth.clone();
        let scopes = self.scopes.clone();

        Box::pin(async move { oauth.token(&scopes).await })
    }
}

pub(crate) fn oauth_grpc<C>(
    channel: tonic::transport::Channel,
    oauth: Option<crate::Auth<C>>,
    scopes: Vec<String>,
) -> AuthGrpcService<tonic::transport::Channel, OAuthTokenSource<C>>
where
    C: hyper::client::connect::Connect + Clone + Send + Sync + 'static,
{
    let token_fn = oauth.map(move |oauth| OAuthTokenSource {
        oauth,
        scopes: Arc::from(scopes),
    });

    AuthGrpcService {
        inner: channel,
        token_fn,
    }
}

/// A [gRPC service](tonic::client::GrpcService) with authorization support.
///
/// Before being sent, each request issued through this service will fetch a token from the given
/// token function, and add that token as an authorization header in the request if a token_fn
/// exists. For testing against an emulator, like GCP PubSub Emulator, the authorization step is
/// skipped.
///
/// ```no_run
/// use ya_gcp::auth::grpc::AuthGrpcService;
/// # #[derive(Copy, Clone)] struct TokenMachine;
/// # impl TokenMachine { async fn get_token(self) -> Result<&'static str, std::io::Error> { Ok("") } }
///
/// // have some source of authorization tokens
/// let token_machine = // ...
/// # TokenMachine;
///
/// # async {
/// // initialize a gRPC connection to the desired end point
/// let connection = tonic::transport::Endpoint::new("https://my.service.endpoint.com")?
///     .connect()
///     .await?;
///
/// // wrap the connection in the auth service with the given token source
/// let service = AuthGrpcService::new(connection, Some(move || {
///     token_machine.get_token()
/// }));
///
/// # struct MyGrpcClient;
/// # impl MyGrpcClient { fn new(_: impl tonic::client::GrpcService<tonic::body::BoxBody>) { } }
/// // issue gRPC calls through this service by submitting it to your generated client's
/// // constructor
/// let my_grpc_client = MyGrpcClient::new(service);
///
/// # Ok::<(), Box<dyn std::error::Error>>(())
///
/// # };
/// ```
#[derive(Debug, Clone)]
pub struct AuthGrpcService<Service, TokenFn> {
    token_fn: Option<TokenFn>,
    inner: Service,
}

impl<Service, TokenFn> AuthGrpcService<Service, TokenFn> {
    /// Wrap the given service to add authorization headers to each request
    pub fn new<ReqBody>(service: Service, token_fn: Option<TokenFn>) -> Self
    where
        // Generic bounds included on the constructor because having them only on the trait impl
        // doesn't produce good compiler diagnostics
        Service: GrpcService<ReqBody> + Clone + 'static,
        Service::Error: std::error::Error + Send + Sync + 'static,
        TokenFn: TokenSource,
    {
        Self {
            inner: service,
            token_fn,
        }
    }
}

/// Errors that may occur when sending authorized gRPC messages
#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
pub enum AuthGrpcError<ServiceErr, TokenErr>
where
    ServiceErr: std::fmt::Debug + std::error::Error + 'static,
    TokenErr: std::fmt::Debug + std::error::Error + 'static,
{
    /// An error in getting a valid auth token
    #[error("Failed to get authorization token")]
    Auth(#[source] TokenErr),

    /// The given auth token was not valid for an HTTP header
    #[error("Auth token formed invalid HTTP header: {1}")]
    InvalidToken(#[source] http::header::InvalidHeaderValue, String),

    /// An error propagated from the underlying service
    #[error(transparent)]
    Grpc(ServiceErr),
}

impl<Service, TokenFn, ReqBody> GrpcService<ReqBody> for AuthGrpcService<Service, TokenFn>
where
    Service: GrpcService<ReqBody> + Clone + Send + 'static,
    Service::Error: std::error::Error + Send + Sync + 'static,
    Service::Future: Send,
    TokenFn: TokenSource,
    ReqBody: Send + 'static,
{
    type Error = AuthGrpcError<Service::Error, TokenFn::Error>;
    // TODO impl-trait-type-alias will allow a concrete type instead of box+dyn
    // (manually implementing a future would work too, but that's tedious and error prone)
    type Future = BoxFuture<'static, Result<http::Response<Self::ResponseBody>, Self::Error>>;
    type ResponseBody = Service::ResponseBody;

    fn poll_ready(&mut self, cx: &mut Context) -> Poll<Result<(), Self::Error>> {
        self.inner.poll_ready(cx).map_err(AuthGrpcError::Grpc)
    }

    fn call(&mut self, mut request: http::Request<ReqBody>) -> Self::Future {
        // start getting the token. This happens outside the async block so that the block doesn't
        // borrow `self`
        let token_fut = self.token_fn.as_ref().map(|token_fn| token_fn.get_token());

        // This clone is necessary because the returned future must be 'static, so it cannot borrow
        // `self` and must instead take ownership. Fortunately tonic::transport::Channel is
        // explcitly documented as being cheap to clone
        let inner = self.inner.clone();

        // take and use `self`'s version of inner, and replace it with the clone.  this is
        // necessary because `poll_ready` was called on the original and not the clone. See
        // https://github.com/tower-rs/tower/pull/548
        let mut inner = std::mem::replace(&mut self.inner, inner);

        Box::pin(async move {
            if let Some(token_fut) = token_fut {
                let token = token_fut.await.map_err(AuthGrpcError::Auth)?;

                crate::auth::add_auth_token(&mut request, &token)
                    .map_err(|e| AuthGrpcError::InvalidToken(e, token.as_ref().to_owned()))?;
            }

            inner.call(request).await.map_err(AuthGrpcError::Grpc)
        })
    }
}

#[cfg(test)]
mod test {
    use super::*;

    /// Test that authorization tokens are attached to requests
    #[tokio::test]
    async fn auth_token_in_request_header() -> Result<(), Box<dyn std::error::Error>> {
        const TOKEN: &str = "this is my token";

        #[derive(Clone)]
        struct AssertionService;

        // Make a mock service that will receive the authorized request and check the token
        impl GrpcService<()> for AssertionService {
            type Error = std::io::Error;
            type Future =
                BoxFuture<'static, Result<http::Response<Self::ResponseBody>, Self::Error>>;
            type ResponseBody = tonic::body::BoxBody;

            fn poll_ready(&mut self, _: &mut Context) -> Poll<Result<(), Self::Error>> {
                unimplemented!()
            }

            fn call(&mut self, request_downstream_of_auth: http::Request<()>) -> Self::Future {
                assert_eq!(
                    request_downstream_of_auth
                        .headers()
                        .get(http::header::AUTHORIZATION),
                    Some(
                        &http::header::HeaderValue::from_str(&format!("Bearer {}", TOKEN)).unwrap()
                    ),
                );
                Box::pin(async { Ok(http::Response::new(tonic::body::empty_body())) })
            }
        }

        let mut auth_service = AuthGrpcService::new(
            AssertionService,
            Some(|| async { Ok::<_, std::io::Error>(TOKEN) }),
        );

        assert!(auth_service
            .call(http::request::Request::new(()))
            .await
            .is_ok());

        Ok(())
    }

    /// Check that errors fetching a token are propagated
    #[tokio::test]
    async fn auth_token_failed_fetch() -> Result<(), Box<dyn std::error::Error>> {
        #[derive(Debug, thiserror::Error)]
        #[error("this is a test error")]
        struct InjectedError;

        let mut auth_service = AuthGrpcService::new(
            tonic::transport::Endpoint::from_static("localhost").connect_lazy(),
            Some(|| async { Err::<String, _>(InjectedError) }),
        );

        assert!(matches!(
            auth_service
                .call(http::request::Request::new(tonic::body::empty_body()))
                .await,
            Err(AuthGrpcError::Auth(InjectedError))
        ));

        Ok(())
    }

    /// Check that errors adding a token as a header are propagated
    #[tokio::test]
    async fn auth_token_invalid_header() -> Result<(), Box<dyn std::error::Error>> {
        #[derive(Debug, thiserror::Error)]
        #[error("this is a test error")]
        struct InjectedError;

        let mut auth_service = AuthGrpcService::new(
            tonic::transport::Endpoint::from_static("localhost").connect_lazy(),
            Some(|| async { Ok::<_, std::io::Error>("\u{0000}") }),
        );

        assert!(matches!(
            auth_service
                .call(http::request::Request::new(tonic::body::empty_body()))
                .await,
            Err(AuthGrpcError::InvalidToken(
                http::header::InvalidHeaderValue { .. },
                _
            ))
        ));

        Ok(())
    }

    /// Check that if no token_fn exists, request does not have an AUTHORIZATION header and
    /// succeeds.
    #[tokio::test]
    async fn no_auth_token_fn() -> Result<(), Box<dyn std::error::Error>> {
        #[derive(Clone)]
        struct OkService;

        // Make a mock service that will receive the authorized request, verify if does not have an
        // AUTHORIZATION and return ok.
        impl GrpcService<()> for OkService {
            type Error = std::io::Error;
            type Future =
                BoxFuture<'static, Result<http::Response<Self::ResponseBody>, Self::Error>>;
            type ResponseBody = tonic::body::BoxBody;

            fn poll_ready(&mut self, _: &mut Context) -> Poll<Result<(), Self::Error>> {
                unimplemented!()
            }

            fn call(&mut self, request_downstream_of_auth: http::Request<()>) -> Self::Future {
                assert_eq!(
                    request_downstream_of_auth
                        .headers()
                        .get(http::header::AUTHORIZATION),
                    None,
                );
                Box::pin(async { Ok(http::Response::new(tonic::body::empty_body())) })
            }
        }

        type TokenFn = dyn Fn() -> futures::future::Ready<Result<String, tonic::transport::Error>>;
        let token_fn: Option<Box<TokenFn>> = None;

        let mut auth_service = AuthGrpcService::new(OkService, token_fn);

        let result = auth_service.call(http::request::Request::new(())).await;

        assert!(result.is_ok());

        Ok(())
    }
}
