use anyhow::Error as AnyhowError;
use axum::Json;
use axum::extract::rejection::JsonRejection;
use axum::response::{Html, IntoResponse};
use http::StatusCode;
use serde::Serialize;
use std::env::VarError;
use std::error::Error as StdError;
use std::fmt;
use tracing::error;
use ts_rs::TS;
use validator::ValidationErrors;

#[derive(Debug, Serialize, TS)]
#[ts(export, export_to = "errors.ts")]
pub enum ErrorType {
    Validation,
    Exception,
    Authentication,
    Authorization,
    NotFound,
}

// A serializable version of ValidationError for ts_rs
#[derive(Debug, Serialize, TS)]
#[ts(export, export_to = "errors.ts")]
pub struct ValidationFieldError {
    pub field: String,
    pub code: String,
    pub message: String,
    pub params: std::collections::HashMap<String, String>,
}

// A serializable version of ValidationErrors for ts_rs
#[derive(Debug, Serialize, TS)]
#[ts(export, export_to = "errors.ts")]
pub struct SerializableValidationErrors {
    pub errors: Vec<ValidationFieldError>,
}

#[derive(Debug, Serialize, TS)]
#[ts(export, export_to = "errors.ts")]
pub struct ErrorResponse {
    pub code: u16,
    pub status: String,
    pub message: String,
    pub user_message: String,
    pub error_type: ErrorType,
    pub validation_errors: Option<SerializableValidationErrors>,
    pub details: Option<String>,
}

#[derive(Clone, Debug)]
pub enum ErrorFormat {
    Html,
    Json,
}

#[derive(Clone, Debug)]
pub enum ErrorKind {
    Validation(ValidationErrors),
    Authentication(),
    Authorization(String),
    NotFound(String),
    Exception(String),
}

#[derive(Debug)]
pub struct AppError {
    pub code: StatusCode,
    pub message: String,
    pub user_message: String,
    pub format: ErrorFormat,
    pub kind: ErrorKind,
    pub source: Option<AnyhowError>,
}

impl fmt::Display for AppError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}: {}", self.code, self.message)
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
    pub fn exception(error: impl Into<String>) -> Self {
        Self {
            message: error.into(),
            user_message: "Internal server error".to_owned(),
            code: StatusCode::INTERNAL_SERVER_ERROR,
            format: ErrorFormat::Html,
            kind: ErrorKind::Exception("".to_owned()),
            source: None,
        }
    }

    pub fn not_found(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
            user_message: "Resource not found".to_owned(),
            code: StatusCode::NOT_FOUND,
            format: ErrorFormat::Html,
            kind: ErrorKind::NotFound("".to_owned()),
            source: None,
        }
    }

    pub fn payload(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
            user_message: "Invalid JSON".to_owned(),
            code: StatusCode::BAD_REQUEST,
            format: ErrorFormat::Json,
            kind: ErrorKind::Exception("".to_owned()),
            source: None,
        }
    }

    pub fn bad_request(message: impl Into<String>) -> Self {
        let message = message.into();
        Self {
            message: message.clone(),
            user_message: message,
            code: StatusCode::BAD_REQUEST,
            format: ErrorFormat::Json,
            kind: ErrorKind::Exception("".to_owned()),
            source: None,
        }
    }

    pub fn validation(errors: ValidationErrors) -> Self {
        Self {
            message: "Validation error".to_owned(),
            user_message: "Validation error".to_owned(),
            code: StatusCode::BAD_REQUEST,
            format: ErrorFormat::Json,
            kind: ErrorKind::Validation(errors),
            source: None,
        }
    }

    pub fn authentication_error() -> Self {
        Self {
            code: StatusCode::UNAUTHORIZED,
            message: "Authentication failed".to_string(),
            user_message: "Please log in to continue".to_string(),
            format: ErrorFormat::Json,
            kind: ErrorKind::Authentication(),
            source: None,
        }
    }

    pub fn authorization_error(details: impl Into<String>) -> Self {
        Self {
            code: StatusCode::FORBIDDEN, // 403 Forbidden
            message: "Authorization failed".to_string(),
            user_message: "You don't have permission to access this resource".to_string(),
            format: ErrorFormat::Json,
            kind: ErrorKind::Authorization(details.into()),
            source: None,
        }
    }

    pub fn with_context<E>(message: impl Into<String>, error: E) -> Self
    where
        E: Into<AnyhowError>,
    {
        Self {
            message: message.into(),
            user_message: "Internal server error".to_owned(),
            code: StatusCode::INTERNAL_SERVER_ERROR,
            format: ErrorFormat::Html,
            kind: ErrorKind::Exception("".to_owned()),
            source: Some(error.into()),
        }
    }

    pub fn with_format(self, format: ErrorFormat) -> Self {
        Self { format, ..self }
    }
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

impl From<AppError> for ErrorResponse {
    fn from(error: AppError) -> Self {
        let (error_type, validation_errors, details) = match &error.kind {
            ErrorKind::Validation(errors) => (
                ErrorType::Validation,
                Some(SerializableValidationErrors::from(errors.clone())),
                None,
            ),
            ErrorKind::Exception(detail) => (ErrorType::Exception, None, Some(detail.clone())),
            ErrorKind::Authentication() => (ErrorType::Authentication, None, None),
            ErrorKind::Authorization(detail) => {
                (ErrorType::Authorization, None, Some(detail.clone()))
            }
            ErrorKind::NotFound(detail) => (ErrorType::NotFound, None, Some(detail.clone())),
        };

        Self {
            code: error.code.as_u16(),
            status: error
                .code
                .canonical_reason()
                .unwrap_or("Unknown")
                .to_string(),
            message: error.message,
            user_message: error.user_message,
            error_type,
            validation_errors,
            details,
        }
    }
}

impl From<ErrorKind> for AppError {
    fn from(kind: ErrorKind) -> Self {
        match kind {
            ErrorKind::Validation(errors) => Self::validation(errors),
            ErrorKind::Authentication() => Self::authentication_error(),
            ErrorKind::Authorization(details) => Self::authorization_error(details),
            ErrorKind::NotFound(msg) => Self::not_found(msg),
            ErrorKind::Exception(msg) => Self::exception(msg),
        }
    }
}

impl From<AnyhowError> for AppError {
    fn from(err: AnyhowError) -> Self {
        AppError::with_context(err.to_string(), err)
    }
}

impl From<JsonRejection> for AppError {
    fn from(err: JsonRejection) -> Self {
        AppError::payload(format!("JSON payload error: {:#}", err))
    }
}

impl From<sqlx::Error> for AppError {
    fn from(err: sqlx::Error) -> Self {
        AppError::exception(format!("Database query error: {:#}", err))
    }
}

impl From<VarError> for AppError {
    fn from(err: VarError) -> Self {
        AppError::exception(format!("Environment variable error: {:#}", err))
    }
}

impl From<ValidationErrors> for AppError {
    fn from(err: ValidationErrors) -> Self {
        AppError::validation(err)
    }
}

impl From<serde_json::Error> for AppError {
    fn from(err: serde_json::Error) -> Self {
        AppError::exception(format!("JSON serialization error: {:#}", err))
    }
}

impl IntoResponse for AppError {
    fn into_response(self) -> axum::response::Response {
        error!("AppError: {}", self.message);

        let status = self.code;
        let format = self.format.clone();
        let user_message = self.message.clone();
        let error_response: ErrorResponse = self.into();

        match format {
            ErrorFormat::Json => (status, Json(error_response)).into_response(),
            ErrorFormat::Html => {
                let html_content = format!(
                    r#"
                  <!DOCTYPE html>
                  <html lang="en">
                  <head>
                      <meta charset="utf-8">
                      <title>Oops!</title>
                  </head>
                  <body>
                      <h1>Oops!</h1>
                      <p>Sorry, but something went wrong.</p>
                      <p>{}</p>
                  </body>
                  </html>
                  "#,
                    user_message
                );

                (status, Html(html_content)).into_response()
            }
        }
    }
}
