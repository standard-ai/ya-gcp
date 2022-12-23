#![deny(
    missing_docs,
    rustdoc::broken_intra_doc_links,
    clippy::all,
    unsafe_code,
    unreachable_pub
)]
#![cfg_attr(not(test), deny(unused))]
#![cfg_attr(docsrs, feature(doc_cfg))]
// nest cfg_attr to save old compilers from failing to parse `doc = macro!()`
#![cfg_attr(docsrs, feature(extended_key_value_attributes), cfg_attr(docsrs, doc = include_str!("../README.md")))]
#![cfg_attr(
    not(docsrs),
    doc = "
    A set of APIs and utilties used to interact with [Google Cloud Platform (GCP)](https://cloud.google.com/) services.

    Use `RUSTDOCFLAGS=\"--cfg docsrs\"` to generate crate-level docs, or read the [readme](README.md)
    "
)]

// re-export the Connect trait because users will need it for the bound on clients
pub use ::hyper::client::connect::Connect;

// a convenience alias
pub(crate) type Auth<C> = yup_oauth2::authenticator::Authenticator<C>;

/// Make a given config struct into a builder using the given default values for fields
macro_rules! config_default {
    (
        $(#[$struct_attr:meta])*
        $struct_vis:vis struct $struct_name:ident {
            $(
                $(#[$field_attr:meta])*
                // Unfortunately there doesn't seem to be a way to pass
                // `stringify!($struct_name::$field_name)` to `#[serde(default = "fn")]` because
                // the proc macro requires a literal and macros like `stringify` are expanded
                // lazily (i.e. after the proc macro expansion). Instead, we require the caller to
                // provide the string themselves
                @default($field_default:expr, $field_default_fn:literal)
                $field_vis:vis $field_name:ident : $field_type:ty,
            )*
        }
    ) => {
        $(#[$struct_attr])*
        $struct_vis struct $struct_name {
            $(
                $(#[$field_attr])*
                #[serde(default = $field_default_fn)]
                #[cfg_attr(docsrs, cfg_attr(docsrs, doc = concat!(
                    "<br>_Default value:_ `", stringify!($field_default), "`"
                )))]
                $field_vis $field_name : $field_type,
            )*
        }

        impl $struct_name {
            /// Construct a new instance with the default values
            pub fn new() -> Self {
                Self::default()
            }

            ::paste::paste! {
                $(
                    #[cfg_attr(docsrs, cfg_attr(docsrs, doc = concat!(
                            "Set [`", stringify!($field_name), "`]",
                            "(#structfield.", stringify!($field_name), ") to the given value"
                        )
                    ))]
                    #[cfg_attr(not(docsrs), doc = "Set this field to the given value")]
                    #[allow(deprecated)]
                    pub fn $field_name(mut self, $field_name: $field_type) -> Self {
                        self.$field_name = $field_name;
                        self
                    }

                    fn [<default_ $field_name>]() -> $field_type {
                        $field_default
                    }
                )*
            }

        }

        impl ::core::default::Default for $struct_name {
            #[cfg_attr(docsrs, cfg_attr(docsrs, doc = concat!(
                "_Default value:_ `", stringify!($struct_name {
                    $(
                        $field_name: $field_default
                    ),*
                }), "`"
            )))]
            fn default() -> $struct_name {
                #[allow(deprecated)]
                $struct_name {
                    $(
                        $field_name: $field_default
                    ),*
                }
            }
        }
    };
}

pub mod auth;

#[cfg(feature = "bigtable")]
#[cfg_attr(docsrs, doc(cfg(feature = "bigtable")))]
pub mod bigtable;

#[cfg(feature = "grpc")]
#[cfg_attr(docsrs, doc(cfg(any(feature = "pubsub", feature = "grpc"))))]
pub mod grpc;

#[cfg(feature = "pubsub")]
#[cfg_attr(docsrs, doc(cfg(feature = "pubsub")))]
pub mod pubsub;

#[cfg(feature = "storage")]
#[cfg_attr(docsrs, doc(cfg(feature = "storage")))]
pub mod storage;

mod builder;
pub use builder::{
    AuthFlow, ClientBuilder, ClientBuilderConfig, CreateBuilderError, DefaultConnector,
    ServiceAccountAuth,
};

pub mod retry_policy;
pub use retry_policy::RetryPolicy;
