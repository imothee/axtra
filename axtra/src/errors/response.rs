//! Response handling and conversion logic for AppError.

use axum::{
    Json,
    response::{Html, IntoResponse, Response},
};
use std::{fs, path::Path};
use tracing::{error, info, warn};

use crate::errors::{AppError, ErrorCode, ErrorFormat, ErrorResponse};

#[cfg(feature = "notify-error-discord")]
use crate::errors::notifiers::discord_notifier;
#[cfg(feature = "notify-error-slack")]
use crate::errors::notifiers::slack_notifier;

macro_rules! notify_critical_error {
    ($self:expr) => {
        #[cfg(feature = "notify-error-slack")]
        $self.send_slack_notification();

        #[cfg(feature = "notify-error-discord")]
        $self.send_discord_notification();

        #[cfg(feature = "sentry")]
        sentry::capture_error(&$self);
    };
}

impl AppError {
    /// Generates a formatted error message for logging and notifications.
    fn formatted_message(&self) -> String {
        let location = self.location();
        let error_code = self.code();
        let message = self.log_message();

        format!("[{location}][{error_code:?}] {message}")
    }

    /// Generates a detailed log message, recursively including sources.
    fn log_message(&self) -> String {
        fn proxy_source(
            source: &Option<Box<dyn std::error::Error + Send + Sync>>,
        ) -> Option<String> {
            source.as_ref().and_then(|src| {
                src.downcast_ref::<AppError>()
                    .map(|app_err| app_err.log_message())
                    .or_else(|| Some(format!("{src:?}")))
            })
        }

        match self {
            AppError::Authentication { .. } => "Authentication failed".to_string(),
            AppError::Authorization {
                resource, action, ..
            } => format!("'{action}' on '{resource}'"),
            AppError::BadRequest { detail, source, .. } => match proxy_source(source) {
                Some(msg) => format!("Bad Request: {detail} | caused by: {msg}"),
                None => detail.to_string(),
            },
            AppError::Database {
                message, source, ..
            } => format!("{message} | sqlx: {source:?}"),
            AppError::Exception { detail, source, .. } => match proxy_source(source) {
                Some(msg) => format!("{detail} | caused by: {msg}"),
                None => detail.to_string(),
            },
            AppError::NotFound { resource, .. } => {
                format!("Resource '{resource}'")
            }
            AppError::Validation { .. } => "Invalid payload".to_string(),
        }
    }

    /// Returns a user-friendly message for the error.
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

    #[cfg(feature = "notify-error-discord")]
    fn send_discord_notification(&self) {
        if let Some(notifier) = discord_notifier() {
            let app_name = std::env::var("APP_NAME").unwrap_or_else(|_| "Rust".to_string());
            let formatted_message = self.formatted_message();

            let embeds = serde_json::json!([
                {
                    "title": format!(":red_circle: Exception — {app_name}"),
                    "color": 16711680, // Red
                    "fields": [
                        {
                            "name": "Details",
                            "value": format!("```{formatted_message}```"),
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

    #[cfg(feature = "notify-error-slack")]
    fn send_slack_notification(&self) {
        if let Some(notifier) = slack_notifier() {
            let app_name = std::env::var("APP_NAME").unwrap_or("Rust".to_string());
            let formatted_message = self.formatted_message();

            let blocks = serde_json::json!([
                {
                    "type": "section",
                    "text": {
                        "type": "mrkdwn",
                        "text": format!(":red_circle: *Exception* — `{app_name}`")
                    }
                },
                {
                    "type": "section",
                    "text": {
                        "type": "mrkdwn",
                        "text": format!("```{formatted_message}```")
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
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let status = self.status_code();
        let format = self.format();
        let error_code = self.code();
        let formatted_message = self.formatted_message();

        // Log the error
        match error_code {
            ErrorCode::Authentication | ErrorCode::Authorization => {
                info!("{formatted_message}");
            }
            ErrorCode::BadRequest | ErrorCode::NotFound | ErrorCode::Validation => {
                warn!("{formatted_message}");
            }
            ErrorCode::Database | ErrorCode::Exception => {
                error!("{formatted_message}");
                notify_critical_error!(self);
            }
        }

        // Generate response
        match format {
            ErrorFormat::Json => {
                let error_response = ErrorResponse {
                    status: status.canonical_reason().unwrap_or("Unknown").to_string(),
                    message: self.user_message().to_string(),
                    code: self.code(),
                    validation_errors: match &self {
                        AppError::Validation { errors, .. } => Some(errors.clone().into()),
                        _ => None,
                    },
                };
                (status, Json(error_response)).into_response()
            }
            ErrorFormat::Html => {
                let file_path = match error_code {
                    ErrorCode::NotFound => "dist/404.html",
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
