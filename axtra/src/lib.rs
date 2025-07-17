//! # Axtra
//!
//! Opinionated helpers for Axum + Astro projects.
//!
//! ## Features
//!
//! - **AppError**: Unified error type for Axum APIs.
//! - **Error Macros**: Ergonomic error construction with `app_error!`.
//! - **TypeScript Type Generation**: Rust error types exported via `ts-rs`.
//! - **Error Notifications**: Sentry, Slack, Discord integration (optional).
//! - **Wrapped JSON Responses**: `WrappedJson<T>` and `ResponseKey` derive macro.
//! - **Health Check Endpoint**: Built-in Axum route for Postgres connectivity.
//! - **Static File Serving**: SPA and static file helpers for Axum.
//! - **Bouncer** (optional): Reject and ban IP's hitting invalid endpoints.
//!
//! ## See Also
//! - [README](https://github.com/imothee/axtra)
//! - [API Docs (docs.rs)](https://docs.rs/axtra)
//! - [Changelog](./CHANGELOG.md)

pub use axtra_macros::*;

#[cfg(feature = "bouncer")]
pub mod bouncer;
pub mod errors;
#[cfg(feature = "notifier")]
pub mod notifier;
pub mod response;
pub mod routes;
