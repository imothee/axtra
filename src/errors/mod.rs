use std::error::Error as StdError;
use std::path::Path;
use std::{fmt, fs};

use anyhow::Error as AnyhowError;
use api::{ErrorResponse, SerializableValidationErrors, ValidationFieldError};
use axum::Json;
use axum::extract::rejection::JsonRejection;
use axum::response::{Html, IntoResponse};
use enums::{ErrorFormat, ErrorKind};
use http::StatusCode;
use tracing::{error, info, warn};
use validator::ValidationErrors;

pub mod api;
pub mod enums;

#[derive(Debug)]
pub struct AppError {
    pub code: StatusCode,
    pub format: ErrorFormat,
    pub kind: ErrorKind,
    pub source: Option<AnyhowError>,
}

/* Implement std::err */
impl fmt::Display for AppError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}: {}", self.code, self.log_message())
    }
}

impl StdError for AppError {
    fn source(&self) -> Option<&(dyn StdError + 'static)> {
        // If you have nested errors, you can return them here
        // For now, returning None as there's no nested error
        None
    }
}

impl AppError {
    pub fn bad_request(detail: impl Into<String>, format: Option<ErrorFormat>) -> Self {
        Self {
            code: StatusCode::BAD_REQUEST,
            format: format.unwrap_or(ErrorFormat::Html),
            kind: ErrorKind::BadRequest {
                detail: detail.into(),
            },
            source: None,
        }
    }

    pub fn internal_server_error(detail: impl Into<String>, format: Option<ErrorFormat>) -> Self {
        Self {
            code: StatusCode::INTERNAL_SERVER_ERROR,
            format: format.unwrap_or(ErrorFormat::Html),
            kind: ErrorKind::Exception {
                detail: detail.into(),
            },
            source: None,
        }
    }

    pub fn not_found(resource: impl Into<String>, format: Option<ErrorFormat>) -> Self {
        Self {
            code: StatusCode::NOT_FOUND,
            format: format.unwrap_or(ErrorFormat::Html),
            kind: ErrorKind::NotFound {
                resource: resource.into(),
            },
            source: None,
        }
    }

    pub fn unauthenticated(format: Option<ErrorFormat>) -> Self {
        Self {
            code: StatusCode::UNAUTHORIZED,
            format: format.unwrap_or(ErrorFormat::Html),
            kind: ErrorKind::Authentication,
            source: None,
        }
    }

    pub fn unauthorized(
        resource: impl Into<String>,
        action: impl Into<String>,
        format: Option<ErrorFormat>,
    ) -> Self {
        Self {
            code: StatusCode::FORBIDDEN,
            format: format.unwrap_or(ErrorFormat::Html),
            kind: ErrorKind::Authorization {
                resource: resource.into(),
                action: action.into(),
            },
            source: None,
        }
    }

    pub fn validation(errors: ValidationErrors, format: Option<ErrorFormat>) -> Self {
        Self {
            code: StatusCode::BAD_REQUEST,
            format: format.unwrap_or(ErrorFormat::Html),
            kind: ErrorKind::Validation { errors },
            source: None,
        }
    }

    pub fn with_context<E>(message: impl Into<String>, error: E) -> Self
    where
        E: Into<AnyhowError>,
    {
        Self {
            code: StatusCode::INTERNAL_SERVER_ERROR,
            format: ErrorFormat::Html,
            kind: ErrorKind::Exception {
                detail: message.into(),
            },
            source: Some(error.into()),
        }
    }
}

impl AppError {
    pub fn json_bad_request(detail: impl Into<String>) -> Self {
        Self::bad_request(detail, Some(ErrorFormat::Json))
    }

    pub fn json_not_found(resource: impl Into<String>) -> Self {
        Self::not_found(resource, Some(ErrorFormat::Json))
    }

    pub fn json_unauthenticated() -> Self {
        Self::unauthenticated(Some(ErrorFormat::Json))
    }

    pub fn json_unauthorized(resource: impl Into<String>, action: impl Into<String>) -> Self {
        Self::unauthorized(resource, action, Some(ErrorFormat::Json))
    }

    pub fn json_validation(errors: ValidationErrors) -> Self {
        Self::validation(errors, Some(ErrorFormat::Json))
    }
}

impl AppError {
    // Helper function to generate the log message
    fn log_message(&self) -> String {
        match &self.kind {
            ErrorKind::BadRequest { detail } => format!("Bad Request: {}", detail),
            ErrorKind::NotFound { resource } => format!("Not Found: Resource '{}'", resource),
            ErrorKind::Authorization { resource, action } => {
                format!("Unauthorized: '{}' on '{}'", action, resource)
            }
            ErrorKind::Exception { detail } => format!("Exception: {}", detail),
            ErrorKind::Validation { .. } => "Validation error occurred".to_string(),
            ErrorKind::Authentication => "Authentication failed".to_string(),
        }
    }
}

/*
* Implicit conversion using from should be limited to avoid heavy dependencies in this library
* We should handle validation errors since they will happen in every post/patch request
* The other option to get explicitly named errors is to use context to bubble a generic exception
* The rest need to be returned explicitly
*/

// Allows us to convert from any anyhow::Error with context directly
impl From<AnyhowError> for AppError {
    fn from(err: AnyhowError) -> Self {
        AppError::with_context(err.to_string(), err)
    }
}

// Allows us to convert from validation errors directly
impl From<ValidationErrors> for AppError {
    fn from(err: ValidationErrors) -> Self {
        AppError::validation(err, Some(ErrorFormat::Json))
    }
}

// Allows us to convert from sqlx errors directly
impl From<sqlx::Error> for AppError {
    fn from(err: sqlx::Error) -> Self {
        AppError::with_context("Database error occurred", err)
    }
}

impl From<JsonRejection> for AppError {
    fn from(err: JsonRejection) -> Self {
        AppError::json_bad_request(format!("JSON payload error: {:#}", err))
    }
}

// Allows us to convert from validation errors directly
// This is used to serialize the validation errors into a format that can be returned in the response
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

impl IntoResponse for AppError {
    fn into_response(self) -> axum::response::Response {
        let status = self.code;
        let format = self.format.clone();

        let log_message = self.log_message();
        let source = self.source.as_ref().map(|e| e.to_string());

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

        let user_message = match &self.kind {
            ErrorKind::BadRequest { detail } => detail,
            ErrorKind::NotFound { .. } => "The requested resource was not found.",
            ErrorKind::Authorization { .. } => "You are not authorized to perform this action.",
            ErrorKind::Exception { .. } => "An internal server error occurred.",
            ErrorKind::Validation { .. } => "There was a validation error with your request.",
            ErrorKind::Authentication => "Authentication is required to access this resource.",
        };

        match format {
            ErrorFormat::Json => {
                let error_response = ErrorResponse {
                    status: status.canonical_reason().unwrap_or("Unknown").to_string(),
                    message: user_message.to_string(),
                    kind: self.kind.clone(), // Serialized as a string (e.g., "Validation")
                    validation_errors: match self.kind {
                        ErrorKind::Validation { errors } => Some(errors.into()),
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
                    // Fallback if the file cannot be read
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
                        user_message
                    )
                });

                (status, Html(html_content)).into_response()
            }
        }
    }
}
