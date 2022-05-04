//! Items and functionality specific to gRPC services

use std::time::Duration;
use tonic::transport::Endpoint;

mod status_code_set;
pub use status_code_set::StatusCodeSet;

config_default! {
    /// Configuration for a gRPC connection to some endpoint.
    ///
    /// These fields generally correspond to the associated setting on the [`Endpoint`] type. If
    /// set to `None`, the configured endpoint will use [`Endpoint`]'s default value for that
    /// setting
    #[derive(Debug, Clone, Eq, PartialEq, Hash, serde::Deserialize)]
    #[non_exhaustive]
    #[allow(missing_docs)]
    pub struct EndpointConfig {
        @default(None, "EndpointConfig::default_concurrency_limit")
        pub concurrency_limit: Option<usize>,

        #[serde(with = "humantime_serde")]
        @default(None, "EndpointConfig::default_connect_timeout")
        pub connect_timeout: Option<Duration>,

        @default(None, "EndpointConfig::default_http2_adaptive_window")
        pub http2_adaptive_window: Option<bool>,

        #[serde(with = "humantime_serde")]
        @default(None, "EndpointConfig::default_http2_keep_alive_interval")
        pub http2_keep_alive_interval: Option<Duration>,

        @default(None, "EndpointConfig::default_initial_connection_window_size")
        pub initial_connection_window_size: Option<u32>,

        @default(None, "EndpointConfig::default_initial_stream_window_size")
        pub initial_stream_window_size: Option<u32>,

        #[serde(with = "humantime_serde")]
        @default(None, "EndpointConfig::default_keep_alive_timeout")
        pub keep_alive_timeout: Option<Duration>,

        @default(None, "EndpointConfig::default_keep_alive_while_idle")
        pub keep_alive_while_idle: Option<bool>,

        #[serde(with = "humantime_serde")]
        @default(None, "EndpointConfig::default_tcp_keepalive")
        pub tcp_keepalive: Option<Duration>,

        @default(None, "EndpointConfig::default_tcp_nodelay")
        pub tcp_nodelay: Option<bool>,

        #[serde(with = "humantime_serde")]
        @default(None, "EndpointConfig::default_timeout")
        pub timeout: Option<Duration>,
    }
}

macro_rules! apply_if_some {
    ($struct:ident, $target:ident:
     $($field:ident $(: $ty_hint:ty)?),*$(,)?
    ) => {
        $(
            if let Some(val) = $struct.$field {
                $target = $target.$field(Into::<$($ty_hint)?>::into(val));
            }
        )*
    }
}

impl EndpointConfig {
    pub(crate) fn apply(self, mut endpoint: Endpoint) -> Endpoint {
        apply_if_some!(
            self,
            endpoint: concurrency_limit,
            connect_timeout,
            http2_adaptive_window,
            http2_keep_alive_interval,
            initial_connection_window_size: Option<u32>,
            initial_stream_window_size: Option<u32>,
            keep_alive_timeout,
            keep_alive_while_idle,
            tcp_keepalive,
            tcp_nodelay,
            timeout,
        );
        endpoint
    }
}
