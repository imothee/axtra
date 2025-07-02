//! Error types and helpers for the application.
//!
//! This module provides:
//! - Validation error serialization
//! - AppError enum for all error types
//! - Error formatting helpers (JSON/HTML)
//! - Logging and response conversion utilities
//! - Macros for error location

// --- Imports ---
use axum::{
    Json,
    extract::rejection::JsonRejection,
    response::{Html, IntoResponse, Response},
};
use http::StatusCode;
use serde::Serialize;
use std::{collections::HashMap, fs, path::Path};
use thiserror::Error;
use tracing::{error, info, warn};
use ts_rs::TS;
use validator::ValidationErrors;

#[cfg(any(feature = "notify-error-slack", feature = "notify-error-discord"))]
use crate::notifier::Notifier;

#[cfg(any(feature = "notify-error-slack", feature = "notify-error-discord"))]
use std::sync::OnceLock;

// --- Macros ---

/// Macro to prepend the current module path and line to a message.
/// Usage: `err_with_loc!("something went wrong: {}", detail)`
#[macro_export]
macro_rules! err_with_loc {
    ($($arg:tt)*) => {
        format!("[{}:{}] {}", module_path!(), line!(), format!($($arg)*))
    };
}

// Notification Clients
#[cfg(feature = "notify-error-slack")]
static SLACK_NOTIFIER: OnceLock<Option<Notifier>> = OnceLock::new();

#[cfg(feature = "notify-error-slack")]
fn slack_notifier() -> Option<&'static Notifier> {
    SLACK_NOTIFIER
        .get_or_init(|| {
            std::env::var("SLACK_ERROR_WEBHOOK_URL")
                .ok()
                .map(Notifier::with_slack)
        })
        .as_ref()
}

#[cfg(feature = "notify-error-discord")]
static DISCORD_NOTIFIER: OnceLock<Option<Notifier>> = OnceLock::new();

#[cfg(feature = "notify-error-discord")]
fn discord_notifier() -> Option<&'static Notifier> {
    DISCORD_NOTIFIER
        .get_or_init(|| {
            std::env::var("DISCORD_ERROR_WEBHOOK_URL")
                .ok()
                .map(Notifier::with_discord)
        })
        .as_ref()
}

// --- Validation Error Serialization ---

/// Represents a single field validation error.
#[derive(Debug, Serialize, TS)]
#[ts(export, export_to = "errors.ts")]
#[serde(rename_all = "camelCase")]
pub struct ValidationFieldError {
    pub field: String,
    pub code: String,
    pub message: String,
    pub params: HashMap<String, String>,
}

/// Represents all validation errors in a serializable form.
#[derive(Debug, Serialize, TS)]
#[ts(export, export_to = "errors.ts")]
pub struct SerializableValidationErrors {
    pub errors: Vec<ValidationFieldError>,
}

impl From<ValidationErrors> for SerializableValidationErrors {
    fn from(errors: ValidationErrors) -> Self {
        let mut field_errors = Vec::new();
        for (field, error_map) in errors.field_errors() {
            for error in error_map {
                let params = error
                    .params
                    .iter()
                    .map(|(k, v)| (k.to_string(), v.to_string()))
                    .collect();
                field_errors.push(ValidationFieldError {
                    field: field.to_string(),
                    code: error.code.to_string(),
                    message: error
                        .message
                        .as_ref()
                        .map(|cow| cow.to_string())
                        .unwrap_or_else(|| format!("Validation failed for {}", field)),
                    params,
                });
            }
        }
        SerializableValidationErrors {
            errors: field_errors,
        }
    }
}

// --- Error Format Enum ---

/// Supported error output formats.
#[derive(Clone, Debug, Serialize)]
pub enum ErrorFormat {
    Html,
    Json,
}

// --- Error Code Enum ---

/// Enum of all possible error codes.
#[derive(Debug, Serialize, TS, Clone, Copy)]
#[ts(export, export_to = "errors.ts")]
#[serde(rename_all = "snake_case")]
pub enum ErrorCode {
    Authentication,
    Authorization,
    BadRequest,
    Database,
    Exception,
    NotFound,
    Validation,
}

// --- AppError Enum ---

/// Main application error type, covering all error cases.
#[derive(Debug, Error)]
pub enum AppError {
    #[error("Bad Request: {detail}")]
    BadRequest {
        detail: String,
        #[source]
        source: Option<Box<dyn std::error::Error + Send + Sync>>,
        format: Option<ErrorFormat>,
    },
    #[error("Not Found: {resource}")]
    NotFound {
        resource: String,
        #[source]
        source: Option<Box<dyn std::error::Error + Send + Sync>>,
        format: Option<ErrorFormat>,
    },
    #[error("Unauthorized: {resource} {action}")]
    Authorization {
        resource: String,
        action: String,
        #[source]
        source: Option<Box<dyn std::error::Error + Send + Sync>>,
        format: Option<ErrorFormat>,
    },
    #[error("Authentication required")]
    Authentication {
        #[source]
        source: Option<Box<dyn std::error::Error + Send + Sync>>,
        format: Option<ErrorFormat>,
    },
    #[error("Validation error")]
    Validation {
        errors: ValidationErrors,
        #[source]
        source: Option<Box<dyn std::error::Error + Send + Sync>>,
        format: Option<ErrorFormat>,
    },
    #[error("Database error: {message}")]
    Database {
        message: String,
        #[source]
        source: Box<sqlx::Error>,
        format: Option<ErrorFormat>,
    },
    #[error("Exception: {detail}")]
    Exception {
        detail: String,
        #[source]
        source: Option<Box<dyn std::error::Error + Send + Sync>>,
        format: Option<ErrorFormat>,
    },
}

// --- Error Response Struct ---

/// Structure for serializing error responses to clients.
#[derive(Debug, Serialize, TS)]
#[ts(export, export_to = "errors.ts")]
#[serde(rename_all = "camelCase")]
pub struct ErrorResponse {
    pub status: String,
    pub message: String,
    pub error: String,
    pub code: ErrorCode,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub validation_errors: Option<SerializableValidationErrors>,
}

// --- AppError Constructors ---

impl AppError {
    /// Create a BadRequest error.
    pub fn bad_request(
        detail: impl Into<String>,
        source: Option<Box<dyn std::error::Error + Send + Sync>>,
    ) -> Self {
        Self::BadRequest {
            detail: detail.into(),
            source,
            format: None,
        }
    }

    /// Create a Database error.
    pub fn database(message: impl Into<String>, source: sqlx::Error) -> Self {
        Self::Database {
            message: message.into(),
            source: Box::new(source),
            format: None,
        }
    }

    /// Create an Exception error.
    pub fn exception(
        detail: impl Into<String>,
        source: Option<Box<dyn std::error::Error + Send + Sync>>,
    ) -> Self {
        Self::Exception {
            detail: detail.into(),
            source,
            format: None,
        }
    }

    /// Create a NotFound error.
    pub fn not_found(resource: impl Into<String>) -> Self {
        Self::NotFound {
            resource: resource.into(),
            source: None,
            format: None,
        }
    }

    /// Create an Unauthenticated error.
    pub fn unauthenticated() -> Self {
        Self::Authentication {
            source: None,
            format: None,
        }
    }

    /// Create an Unauthorized error.
    pub fn unauthorized(resource: impl Into<String>, action: impl Into<String>) -> Self {
        Self::Authorization {
            resource: resource.into(),
            action: action.into(),
            source: None,
            format: None,
        }
    }

    /// Create a Validation error.
    pub fn validation(errors: ValidationErrors) -> Self {
        Self::Validation {
            errors,
            source: None,
            format: None,
        }
    }
}

// --- Format Helpers (JSON/HTML) ---

impl AppError {
    // --- JSON helpers ---

    /// Create a JSON BadRequest error.
    pub fn json_bad_request(
        detail: impl Into<String>,
        source: Option<Box<dyn std::error::Error + Send + Sync>>,
    ) -> Self {
        AppError::BadRequest {
            detail: detail.into(),
            source,
            format: Some(ErrorFormat::Json),
        }
    }

    /// Create a JSON Database error.
    pub fn json_database(message: impl Into<String>, source: sqlx::Error) -> Self {
        AppError::Database {
            message: message.into(),
            source: Box::new(source),
            format: Some(ErrorFormat::Json),
        }
    }

    /// Create a JSON Exception error.
    pub fn json_exception(
        detail: impl Into<String>,
        source: Option<Box<dyn std::error::Error + Send + Sync>>,
    ) -> Self {
        AppError::Exception {
            detail: detail.into(),
            source,
            format: Some(ErrorFormat::Json),
        }
    }

    /// Create a JSON NotFound error.
    pub fn json_not_found(resource: impl Into<String>) -> Self {
        AppError::NotFound {
            resource: resource.into(),
            source: None,
            format: Some(ErrorFormat::Json),
        }
    }

    /// Create a JSON Unauthenticated error.
    pub fn json_unauthenticated() -> Self {
        AppError::Authentication {
            source: None,
            format: Some(ErrorFormat::Json),
        }
    }

    /// Create a JSON Unauthorized error.
    pub fn json_unauthorized(resource: impl Into<String>, action: impl Into<String>) -> Self {
        AppError::Authorization {
            resource: resource.into(),
            action: action.into(),
            source: None,
            format: Some(ErrorFormat::Json),
        }
    }

    /// Create a JSON Validation error.
    pub fn json_validation(errors: ValidationErrors) -> Self {
        AppError::Validation {
            errors,
            source: None,
            format: Some(ErrorFormat::Json),
        }
    }

    // --- HTML helpers ---

    /// Create an HTML BadRequest error.
    pub fn html_bad_request(
        detail: impl Into<String>,
        source: Option<Box<dyn std::error::Error + Send + Sync>>,
    ) -> Self {
        AppError::BadRequest {
            detail: detail.into(),
            source,
            format: Some(ErrorFormat::Html),
        }
    }

    /// Create an HTML Database error.
    pub fn html_database(message: impl Into<String>, source: sqlx::Error) -> Self {
        AppError::Database {
            message: message.into(),
            source: Box::new(source),
            format: Some(ErrorFormat::Html),
        }
    }

    /// Create an HTML Exception error.
    pub fn html_exception(
        detail: impl Into<String>,
        source: Option<Box<dyn std::error::Error + Send + Sync>>,
    ) -> Self {
        AppError::Exception {
            detail: detail.into(),
            source,
            format: Some(ErrorFormat::Html),
        }
    }

    /// Create an HTML NotFound error.
    pub fn html_not_found(resource: impl Into<String>) -> Self {
        AppError::NotFound {
            resource: resource.into(),
            source: None,
            format: Some(ErrorFormat::Html),
        }
    }

    /// Create an HTML Unauthenticated error.
    pub fn html_unauthenticated() -> Self {
        AppError::Authentication {
            source: None,
            format: Some(ErrorFormat::Html),
        }
    }

    /// Create an HTML Unauthorized error.
    pub fn html_unauthorized(resource: impl Into<String>, action: impl Into<String>) -> Self {
        AppError::Authorization {
            resource: resource.into(),
            action: action.into(),
            source: None,
            format: Some(ErrorFormat::Html),
        }
    }

    /// Create an HTML Validation error.
    pub fn html_validation(errors: ValidationErrors) -> Self {
        AppError::Validation {
            errors,
            source: None,
            format: Some(ErrorFormat::Html),
        }
    }

    /// Convert any AppError into a JSON-formatted error (drops source if not clonable).
    pub fn as_json(self) -> Self {
        match self {
            AppError::BadRequest { detail, .. } => AppError::BadRequest {
                detail,
                source: None,
                format: Some(ErrorFormat::Json),
            },
            AppError::NotFound { resource, .. } => AppError::NotFound {
                resource,
                source: None,
                format: Some(ErrorFormat::Json),
            },
            AppError::Authorization {
                resource, action, ..
            } => AppError::Authorization {
                resource,
                action,
                source: None,
                format: Some(ErrorFormat::Json),
            },
            AppError::Authentication { .. } => AppError::Authentication {
                source: None,
                format: Some(ErrorFormat::Json),
            },
            AppError::Validation { errors, .. } => AppError::Validation {
                errors,
                source: None,
                format: Some(ErrorFormat::Json),
            },
            AppError::Database { message, .. } => AppError::Database {
                message,
                source: Box::new(sqlx::Error::Protocol("source omitted".into())),
                format: Some(ErrorFormat::Json),
            },
            AppError::Exception { detail, .. } => AppError::Exception {
                detail,
                source: None,
                format: Some(ErrorFormat::Json),
            },
        }
    }
}

// --- AppError Utility Methods ---

impl AppError {
    /// Returns the error code as an enum.
    pub fn code(&self) -> ErrorCode {
        match self {
            AppError::Authentication { .. } => ErrorCode::Authentication,
            AppError::Authorization { .. } => ErrorCode::Authorization,
            AppError::BadRequest { .. } => ErrorCode::BadRequest,
            AppError::Database { .. } => ErrorCode::Database,
            AppError::Exception { .. } => ErrorCode::Exception,
            AppError::NotFound { .. } => ErrorCode::NotFound,
            AppError::Validation { .. } => ErrorCode::Validation,
        }
    }

    /// Generates a detailed log message, recursively including sources.
    fn log_message(&self) -> String {
        fn proxy_source(
            source: &Option<Box<dyn std::error::Error + Send + Sync>>,
        ) -> Option<String> {
            source.as_ref().and_then(|src| {
                // Try to downcast to AppError for recursive log_message
                src.downcast_ref::<AppError>()
                    .map(|app_err| app_err.log_message())
                    .or_else(|| Some(format!("{:?}", src)))
            })
        }

        match self {
            AppError::Authentication { source, .. } => match proxy_source(source) {
                Some(msg) => format!("Authentication failed | caused by: {}", msg),
                None => "Authentication failed".to_string(),
            },
            AppError::Authorization {
                resource,
                action,
                source,
                ..
            } => match proxy_source(source) {
                Some(msg) => format!(
                    "Unauthorized: '{}' on '{}' | caused by: {}",
                    action, resource, msg
                ),
                None => format!("Unauthorized: '{}' on '{}'", action, resource),
            },
            AppError::BadRequest { detail, source, .. } => match proxy_source(source) {
                Some(msg) => format!("Bad Request: {} | caused by: {}", detail, msg),
                None => format!("Bad Request: {}", detail),
            },
            AppError::Database {
                message, source, ..
            } => {
                format!("Database error: {} | sqlx: {:?}", message, source)
            }
            AppError::Exception { detail, source, .. } => match proxy_source(source) {
                Some(msg) => format!("Exception: {} | caused by: {}", detail, msg),
                None => format!("Exception: {}", detail),
            },
            AppError::NotFound {
                resource, source, ..
            } => match proxy_source(source) {
                Some(msg) => format!("Not Found: Resource '{}' | caused by: {}", resource, msg),
                None => format!("Not Found: Resource '{}'", resource),
            },
            AppError::Validation { source, .. } => match proxy_source(source) {
                Some(msg) => format!("Validation error occurred | caused by: {}", msg),
                None => "Validation error occurred".to_string(),
            },
        }
    }

    /// Returns the error format (if set).
    fn format(&self) -> Option<ErrorFormat> {
        match self {
            AppError::Authentication { format, .. }
            | AppError::Authorization { format, .. }
            | AppError::BadRequest { format, .. }
            | AppError::Database { format, .. }
            | AppError::Exception { format, .. }
            | AppError::NotFound { format, .. }
            | AppError::Validation { format, .. } => format.clone(),
        }
    }

    /// Returns the HTTP status code for the error.
    fn status_code(&self) -> StatusCode {
        match self {
            AppError::Authentication { .. } => StatusCode::UNAUTHORIZED,
            AppError::Authorization { .. } => StatusCode::FORBIDDEN,
            AppError::BadRequest { .. } | AppError::Validation { .. } => StatusCode::BAD_REQUEST,
            AppError::Database { .. } | AppError::Exception { .. } => {
                StatusCode::INTERNAL_SERVER_ERROR
            }
            AppError::NotFound { .. } => StatusCode::NOT_FOUND,
        }
    }

    // Returns a user-friendly message for the error.
    fn user_message(&self) -> &str {
        match self {
            AppError::Authentication { .. } => {
                "Authentication is required to access this resource."
            }
            AppError::Authorization { .. } => "You are not authorized to perform this action.",
            AppError::BadRequest { detail, .. } => detail,
            AppError::Database { .. } => "A database error occurred.",
            AppError::Exception { .. } => "An internal server error occurred.",
            AppError::NotFound { .. } => "The requested resource was not found.",
            AppError::Validation { .. } => "There was a validation error with your request.",
        }
    }
}

// --- Error Conversion Implementations ---

/// Converts Axum JSON rejections into AppError.
impl From<JsonRejection> for AppError {
    fn from(err: JsonRejection) -> Self {
        AppError::json_bad_request("Invalid JSON request", Some(Box::new(err)))
    }
}

/// Converts validator errors into AppError.
impl From<ValidationErrors> for AppError {
    fn from(err: ValidationErrors) -> Self {
        AppError::json_validation(err)
    }
}

// --- IntoResponse Implementation ---

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let status = self.status_code();
        let format = self.format().unwrap_or(ErrorFormat::Html);

        let log_message = self.log_message();
        let source = match &self {
            AppError::Authentication { source, .. }
            | AppError::Authorization { source, .. }
            | AppError::BadRequest { source, .. }
            | AppError::Exception { source, .. }
            | AppError::NotFound { source, .. }
            | AppError::Validation { source, .. } => source.as_ref().map(|e| format!("{:?}", e)),
            AppError::Database { source, .. } => Some(format!("{:?}", source)),
        };

        match status {
            StatusCode::BAD_REQUEST | StatusCode::FORBIDDEN => {
                warn!(
                    "AppError: {} (status: {}, source: {:?})",
                    log_message, status, source
                );
            }
            StatusCode::INTERNAL_SERVER_ERROR => {
                error!(
                    "AppError: {} (status: {}, source: {:?})",
                    log_message, status, source
                );
            }
            _ => {
                info!(
                    "AppError: {} (status: {}, source: {:?})",
                    log_message, status, source
                );
            }
        }

        // --- Sentry integration ---
        #[cfg(feature = "sentry")]
        {
            match &self {
                AppError::Database { .. } | AppError::Exception { .. } => {
                    sentry::capture_error(&self);
                }
                _ => {}
            }
        }

        #[cfg(feature = "notify-error-slack")]
        {
            match &self {
                AppError::Database { .. } | AppError::Exception { .. } => {
                    if let Some(notifier) = slack_notifier() {
                        let app_name = std::env::var("APP_NAME").unwrap_or("Rust".to_string());
                        let blocks = serde_json::json!(
                        [
                            {
                                "type": "section",
                                "text": {
                                    "type": "mrkdwn",
                                    "text": format!(":red_circle: *Exception* — `{}`", app_name)
                                }
                            },
                            {
                                "type": "section",
                                "text": {
                                    "type": "mrkdwn",
                                    "text": format!("```{}```", self.log_message())
                                }
                            },
                            {
                                "type": "context",
                                "elements": [
                                    {
                                        "type": "mrkdwn",
                                        "text": "@oncall"
                                    }
                                ]
                            }
                        ]);
                        tokio::spawn(async move {
                            let _ = notifier.notify_slack_rich(blocks).await;
                        });
                    }
                }
                _ => {}
            }
        }

        #[cfg(feature = "notify-error-discord")]
        {
            match &self {
                AppError::Database { .. } | AppError::Exception { .. } => {
                    if let Some(notifier) = discord_notifier() {
                        let app_name =
                            std::env::var("APP_NAME").unwrap_or_else(|_| "Rust".to_string());
                        let embeds = serde_json::json!([
                            {
                                "title": format!(":red_circle: Exception — {}", app_name),
                                "color": 16711680, // Red
                                "fields": [
                                    {
                                        "name": "Details",
                                        "value": format!("```{}```", self.log_message()),
                                        "inline": false
                                    },
                                    {
                                        "name": "\u{200B}",
                                        "value": "@oncall",
                                        "inline": false
                                    }
                                ]
                            }
                        ]);
                        tokio::spawn(async move {
                            let _ = notifier.notify_discord_rich(embeds).await;
                        });
                    }
                }
                _ => {}
            }
        }

        match format {
            ErrorFormat::Json => {
                let error_response = ErrorResponse {
                    status: status.canonical_reason().unwrap_or("Unknown").to_string(),
                    message: self.user_message().to_string(),
                    error: self.to_string(),
                    code: self.code(),
                    validation_errors: match &self {
                        AppError::Validation { errors, .. } => Some(errors.clone().into()),
                        _ => None,
                    },
                };
                (status, Json(error_response)).into_response()
            }
            ErrorFormat::Html => {
                let file_path = match status {
                    StatusCode::NOT_FOUND => "dist/404.html",
                    _ => "dist/500.html",
                };

                let html_content = fs::read_to_string(Path::new(file_path)).unwrap_or_else(|_| {
                    format!(
                        r#"
                        <!DOCTYPE html>
                        <html lang="en">
                        <head>
                            <meta charset="utf-8">
                            <title>Error</title>
                        </head>
                        <body>
                            <h1>Error</h1>
                            <p>{}</p>
                        </body>
                        </html>
                        "#,
                        self.user_message()
                    )
                });

                (status, Html(html_content)).into_response()
            }
        }
    }
}
