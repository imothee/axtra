//! # axtra::bouncer
//!
//! Simple IP banning and malicious path filtering middleware for Tower.
//!
//! ## Overview
//!
//! The `bouncer` module provides middleware for:
//! - Automatically banning IP addresses that hit known malicious or unwanted paths.
//! - Blocking requests to preset or custom path rulesets.
//! - Configurable ban duration, response status, and response body for banned/blocked requests.
//! - Configurable log level for tracing blocked and banned events.
//! - Observability: expose the banlist for monitoring.
//!
//! ## Features
//!
//! - Ban IPs for a configurable duration when they access blocked paths.
//! - Use presets (e.g., "wordpress", "php", "config") or custom paths for filtering.
//! - Customize HTTP status and body for banned and blocked responses.
//! - Set log level for event tracing (`trace`, `debug`, `info`, etc).
//! - Expose the banlist for observability and monitoring.
//!
//! ## Usage Example
//!
//! ```rust
//! use axtra::bouncer::{BouncerConfig, BouncerLayer};
//! use axum::{Router, routing::get};
//! use axum::http::StatusCode;
//! use tracing::Level;
//! use std::time::Duration;
//!
//! // Create a config with presets and custom paths, and customize responses/logging
//! let config = BouncerConfig::from_rules(
//!     &["wordpress", "config"],
//!     &["/custom"]
//! )
//!     .duration(Duration::from_secs(1800))
//!     .banned_response(StatusCode::UNAUTHORIZED)
//!     .blocked_response(StatusCode::NOT_FOUND)
//!     .log_level(Level::INFO);
//! let layer = BouncerLayer::new(config);
//!
//! let app = Router::new()
//!     .route("/", get(|| async { "Hello" }))
//!     .layer(layer);
//! ```
//!
//! ## Presets
//!
//! Available presets for common hacker/scanner paths:
//! - `"wordpress"`
//! - `"php"`
//! - `"config"`
//!
//! ## Advanced Usage
//!
//! You can also pass only presets or only custom paths:
//! ```rust
//! let config = BouncerConfig::from_preset_rules(&["wordpress"]);
//! let config = BouncerConfig::from_custom_rules(&["/admin", "/hidden"]);
//! ```
//!
//! ## Re-exports
//!
//! - [`BouncerConfig`]: Configuration for the bouncer middleware.
//! - [`BouncerLayer`]: Axum layer for IP banning and path filtering.
//!
//! See the README and docs.rs for more details.

mod layer;
mod rules;

pub use layer::{BouncerConfig, BouncerLayer};
