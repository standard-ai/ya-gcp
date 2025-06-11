# Yet Another Google Cloud Platform Crate

[![Build status](https://github.com/standard-ai/ya-gcp/actions/workflows/ci.yml/badge.svg)](https://github.com/standard-ai/ya-gcp/actions)
[![Crate](https://img.shields.io/crates/v/ya-gcp)](https://crates.io/crates/ya-gcp)
[![Docs](https://img.shields.io/docsrs/ya-gcp/latest)](https://docs.rs/ya-gcp)
![License](https://img.shields.io/crates/l/ya-gcp.svg)

ya-gcp provides a set of APIs and utilties used to interact with [Google Cloud
Platform (GCP)](https://cloud.google.com/) services.

## Currently Supported Services
**Production maturity**:
- PubSub

**Alpha maturity**:
- Google Cloud Storage
- Bigtable

Different service APIs can be accessed through modules enabled with
compile-time features. See the list of supported features below. Service
clients are created using the [`ClientBuilder`](crate::builder::ClientBuilder),
which serves as an entry-point to this library.

## Feature Flags
The following flags can be enabled to change what code is included

Services:
- `pubsub` enables the PubSub API
- `storage` enables the GCS API
- `bigtable` enables the Bigtable API

Miscellaneous:
- `rustls` use Rustls for TLS support, enabled by default
- `openssl` use OpenSSL for TLS support
- `emulators` includes support for service emulation (can be useful for testing)

## Comparison to other crates

Generally speaking, this crate aims to provide ergonomic and robust interfaces
for the supported services out-of-the-box. For example, authentication handling
should be simple, with the user only having to provide credentials and not call
out to a separate library. Similarly, idiomatic rust traits should be provided,
such as `Stream` and `Sink` for PubSub subscribing and publishing. Other crates
for interacting with GCP may provide different trade-offs, such as supporting a
greater breadth of services

- [cloud-storage](https://crates.io/crates/cloud-storage) - A library which
	provides access to Google Cloud Storage specifically. While its feature set
	is good, it is not particularly flexible. It doesn't support alternative HTTP
	clients, or different authentication flows.
- [tame-gcs](https://crates.io/crates/tame-gcs) - A library which enables
	accessing GCS, but does not provide a means of performing IO itself.
	`ya-gcp` internally uses `tame-gcs` for its GCS support (providing the
	IO layer)
- [google-cloud](https://crates.io/crates/google-cloud) - A library with a
	similar structure and philosophy to `ya-gcp`. Supports a few more
	services, though support is sometimes in less depth (e.g. doesn't have
	streaming pull requests and reconnection for PubSub)
- [google-pubsub1](https://crates.io/crates/google-pubsub1) - This crate, like
	others from the same generator system in google-apis-rs, provides relatively
	low-level bindings to the services they connect to. The user is responsible
	for bringing their own HTTP client and authentication, making it difficult to
	use. That said, the generated crates do thoroughly cover *many* google
	services
- [google-cloud-rust](https://github.com/mozilla-services/google-cloud-rust) -
	As of this writing still largely experimental. Based on grpcio, which wraps
	the C gRPC library, whereas `ya-gcp` is based on `tonic` in Rust
