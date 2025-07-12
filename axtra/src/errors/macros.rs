//! Error handling macros for axum routes

/// Macro to get the current module and line location.
/// Usage: `error_location!("something went wrong: {}", detail)`
#[macro_export]
macro_rules! error_location {
    () => {
        format!("{}:{}", module_path!(), line!())
    };
}

/// Error macro - handles all error types with optional format
///
/// Usage:
/// - `app_error!(bad_request, json, "Invalid data: {}", field)`
/// - `app_error!(bad_request, with_error, "Invalid data")`
/// - `app_error!(db, "Failed to connect")`
/// - `app_error!(db, json, "Failed to connect")`
/// - `app_error!(not_found, "User not found")`
/// - `app_error!(exception, "Unexpected error")`
/// - `app_error!(unauthenticated)`
/// - `app_error!(unauthorized, "users", "delete")`
/// - `app_error!(validation, errors)`
#[macro_export]
macro_rules! app_error {
    // Bad Request errors
    (bad_request, $msg:expr) => {
        $crate::errors::AppError::bad_request(
            $msg,
            None,
            $crate::error_location!(),
            $crate::errors::ErrorFormat::Html
        )
    };
    (bad_request, json, $msg:expr) => {
        $crate::errors::AppError::bad_request(
            $msg,
            None,
            $crate::error_location!(),
            $crate::errors::ErrorFormat::Json
        )
    };
    (bad_request, html, $msg:expr) => {
        $crate::errors::AppError::bad_request(
            $msg,
            None,
            $crate::error_location!(),
            $crate::errors::ErrorFormat::Html
        )
    };

    // Bad Request with underlying error (returns closure for map_err)
    (bad_request, with_error, $msg:expr) => {
        |e| $crate::errors::AppError::bad_request(
            $msg,
            Some(Box::new(e) as Box<dyn std::error::Error + Send + Sync>),
            $crate::error_location!(),
            $crate::errors::ErrorFormat::Html
        )
    };
    (bad_request, json, with_error, $msg:expr) => {
        |e| $crate::errors::AppError::bad_request(
            $msg,
            Some(Box::new(e) as Box<dyn std::error::Error + Send + Sync>),
            $crate::error_location!(),
            $crate::errors::ErrorFormat::Json
        )
    };
    (bad_request, html, with_error, $msg:expr) => {
        |e| $crate::errors::AppError::bad_request(
            $msg,
            Some(Box::new(e) as Box<dyn std::error::Error + Send + Sync>),
            $crate::error_location!(),
            $crate::errors::ErrorFormat::Html
        )
    };

    // Bad Request with format args (no source)
    (bad_request, $fmt:literal $(, $args:expr)*) => {
        $crate::errors::AppError::bad_request(
            format!($fmt $(, $args)*),
            None,
            $crate::error_location!(),
            $crate::errors::ErrorFormat::Html
        )
    };
    (bad_request, json, $fmt:literal $(, $args:expr)*) => {
        $crate::errors::AppError::bad_request(
            format!($fmt $(, $args)*),
            None,
            $crate::error_location!(),
            $crate::errors::ErrorFormat::Json
        )
    };
    (bad_request, html, $fmt:literal $(, $args:expr)*) => {
        $crate::errors::AppError::bad_request(
            format!($fmt $(, $args)*),
            None,
            $crate::error_location!(),
            $crate::errors::ErrorFormat::Html
        )
    };

    // Bad Request with format args and underlying error (returns closure)
     (bad_request, with_error, $fmt:literal $(, $args:expr)*) => {
        |e| $crate::errors::AppError::bad_request(
            format!($fmt $(, $args)*),
            Some(Box::new(e) as Box<dyn std::error::Error + Send + Sync>),
            $crate::error_location!(),
            $crate::errors::ErrorFormat::Html
        )
    };
    (bad_request, json, with_error, $fmt:literal $(, $args:expr)*) => {
        |e| $crate::errors::AppError::bad_request(
            format!($fmt $(, $args)*),
            Some(Box::new(e) as Box<dyn std::error::Error + Send + Sync>),
            $crate::error_location!(),
            $crate::errors::ErrorFormat::Json
        )
    };
    (bad_request, html, with_error, $fmt:literal $(, $args:expr)*) => {
        |e| $crate::errors::AppError::bad_request(
            format!($fmt $(, $args)*),
            Some(Box::new(e) as Box<dyn std::error::Error + Send + Sync>),
            $crate::error_location!(),
            $crate::errors::ErrorFormat::Html
        )
    };

    // Database errors
    (db, $msg:expr) => {
        |e| $crate::errors::AppError::database(
            $msg,
            e,
            $crate::error_location!(),
            $crate::errors::ErrorFormat::Html
        )
    };
    (db, json, $msg:expr) => {
        |e| $crate::errors::AppError::database(
            $msg,
            e,
            $crate::error_location!(),
            $crate::errors::ErrorFormat::Json
        )
    };
    (db, html, $msg:expr) => {
        |e| $crate::errors::AppError::database(
            $msg,
            e,
            $crate::error_location!(),
            $crate::errors::ErrorFormat::Html
        )
    };

    // Database errors with format args
    (db, $fmt:literal $(, $args:expr)*) => {
        |e| $crate::errors::AppError::database(
            format!($fmt $(, $args)*),
            e,
            $crate::error_location!(),
            $crate::errors::ErrorFormat::Html
        )
    };
    (db, json, $fmt:literal $(, $args:expr)*) => {
        |e| $crate::errors::AppError::database(
            format!($fmt $(, $args)*),
            e,
            $crate::error_location!(),
            $crate::errors::ErrorFormat::Json
        )
    };
    (db, html, $fmt:literal $(, $args:expr)*) => {
        |e| $crate::errors::AppError::database(
            format!($fmt $(, $args)*),
            e,
            $crate::error_location!(),
            $crate::errors::ErrorFormat::Html
        )
    };

    // Exception errors
    (exception, $msg:expr) => {
        |e| $crate::errors::AppError::exception(
            $msg,
            Some(Box::new(e) as Box<dyn std::error::Error + Send + Sync>),
            $crate::error_location!(),
            $crate::errors::ErrorFormat::Html
        )
    };
    (exception, json, $msg:expr) => {
        |e| $crate::errors::AppError::exception(
            $msg,
            Some(Box::new(e) as Box<dyn std::error::Error + Send + Sync>),
            $crate::error_location!(),
            $crate::errors::ErrorFormat::Json
        )
    };
    (exception, html, $msg:expr) => {
        |e| $crate::errors::AppError::exception(
            $msg,
            Some(Box::new(e) as Box<dyn std::error::Error + Send + Sync>),
            $crate::error_location!(),
            $crate::errors::ErrorFormat::Html
        )
    };

    // Exception errors with format args
    (exception, $fmt:literal $(, $args:expr)*) => {
        |e| $crate::errors::AppError::exception(
            format!($fmt $(, $args)*),
            Some(Box::new(e) as Box<dyn std::error::Error + Send + Sync>),
            $crate::error_location!(),
            $crate::errors::ErrorFormat::Html
        )
    };
    (exception, json, $fmt:literal $(, $args:expr)*) => {
        |e| $crate::errors::AppError::exception(
            format!($fmt $(, $args)*),
            Some(Box::new(e) as Box<dyn std::error::Error + Send + Sync>),
            $crate::error_location!(),
            $crate::errors::ErrorFormat::Json
        )
    };
    (exception, html, $fmt:literal $(, $args:expr)*) => {
        |e| $crate::errors::AppError::exception(
            format!($fmt $(, $args)*),
            Some(Box::new(e) as Box<dyn std::error::Error + Send + Sync>),
            $crate::error_location!(),
            $crate::errors::ErrorFormat::Html
        )
    };

    // Throw errors
    (throw, $msg:expr) => {
        $crate::errors::AppError::exception(
            $msg,
            None,
            $crate::error_location!(),
            $crate::errors::ErrorFormat::Html
        )
    };
    (throw, json, $msg:expr) => {
        $crate::errors::AppError::exception(
            $msg,
            None,
            $crate::error_location!(),
            $crate::errors::ErrorFormat::Json
        )
    };
    (throw, html, $msg:expr) => {
        $crate::errors::AppError::exception(
            $msg,
            None,
            $crate::error_location!(),
            $crate::errors::ErrorFormat::Html
        )
    };

    // Throw with format args
    (throw, $fmt:literal $(, $args:expr)*) => {
        $crate::errors::AppError::exception(
            format!($fmt $(, $args)*),
            None,
            $crate::error_location!(),
            $crate::errors::ErrorFormat::Html
        )
    };
    (throw, json, $fmt:literal $(, $args:expr)*) => {
        $crate::errors::AppError::exception(
            format!($fmt $(, $args)*),
            None,
            $crate::error_location!(),
            $crate::errors::ErrorFormat::Json
        )
    };
    (throw, html, $fmt:literal $(, $args:expr)*) => {
        $crate::errors::AppError::exception(
            format!($fmt $(, $args)*),
            None,
            $crate::error_location!(),
            $crate::errors::ErrorFormat::Html
        )
    };

    // Not Found errors
    (not_found, $resource:expr) => {
        $crate::errors::AppError::not_found(
            $resource,
            $crate::error_location!(),
            $crate::errors::ErrorFormat::Html
        )
    };
    (not_found, json, $resource:expr) => {
        $crate::errors::AppError::not_found(
            $resource,
            $crate::error_location!(),
            $crate::errors::ErrorFormat::Json
        )
    };
    (not_found, html, $resource:expr) => {
        $crate::errors::AppError::not_found(
            $resource,
            $crate::error_location!(),
            $crate::errors::ErrorFormat::Html
        )
    };

    // Not Found with format args
    (not_found, $fmt:literal $(, $args:expr)*) => {
        $crate::errors::AppError::not_found(
            format!($fmt $(, $args)*),
            $crate::error_location!(),
            $crate::errors::ErrorFormat::Html
        )
    };
    (not_found, json, $fmt:literal $(, $args:expr)*) => {
        $crate::errors::AppError::not_found(
            format!($fmt $(, $args)*),
            $crate::error_location!(),
            $crate::errors::ErrorFormat::Json
        )
    };
    (not_found, html, $fmt:literal $(, $args:expr)*) => {
        $crate::errors::AppError::not_found(
            format!($fmt $(, $args)*),
            $crate::error_location!(),
            $crate::errors::ErrorFormat::Html
        )
    };

    // Unauthorized errors (need resource and action)
     (unauthorized, $resource:expr, $action:expr) => {
        $crate::errors::AppError::unauthorized(
            $resource,
            $action,
            $crate::error_location!(),
            $crate::errors::ErrorFormat::Html
        )
    };
    (unauthorized, json, $resource:expr, $action:expr) => {
        $crate::errors::AppError::unauthorized(
            $resource,
            $action,
            $crate::error_location!(),
            $crate::errors::ErrorFormat::Json
        )
    };
    (unauthorized, html, $resource:expr, $action:expr) => {
        $crate::errors::AppError::unauthorized(
            $resource,
            $action,
            $crate::error_location!(),
            $crate::errors::ErrorFormat::Html
        )
    };

    // Unauthenticated errors
    (unauthenticated) => {
        $crate::errors::AppError::unauthenticated(
            $crate::error_location!(),
            $crate::errors::ErrorFormat::Html
        )
    };
    (unauthenticated, json) => {
        $crate::errors::AppError::unauthenticated(
            $crate::error_location!(),
            $crate::errors::ErrorFormat::Json
        )
    };
    (unauthenticated, html) => {
        $crate::errors::AppError::unauthenticated(
            $crate::error_location!(),
            $crate::errors::ErrorFormat::Html
        )
    };

    // Validation errors
    (validation, $errors:expr) => {
        $crate::errors::AppError::validation(
            $errors,
            $crate::error_location!(),
            $crate::errors::ErrorFormat::Html
        )
    };
    (validation, json, $errors:expr) => {
        $crate::errors::AppError::validation(
            $errors,
            $crate::error_location!(),
            $crate::errors::ErrorFormat::Json
        )
    };
    (validation, html, $errors:expr) => {
        $crate::errors::AppError::validation(
            $errors,
            $crate::error_location!(),
            $crate::errors::ErrorFormat::Html
        )
    };
}
