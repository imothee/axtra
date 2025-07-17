# Changelog

## 0.2.1

- Added Bouncer module
- Removed errant info in static file serving

## 0.2.0

- Introduced `app_error!` macros for ergonomic error construction.
- Support for error modifiers: `json`, `html`, `with_error`, and format args.
- Closure-based error mapping for `.map_err()` and underlying error propagation.
- Improved error location tracking and logging.
- Expanded macro documentation and examples.

## 0.1.0

- Initial launch.
- Unified `AppError` enum for Axum APIs.
- TypeScript type generation for error codes and responses.
- `WrappedJson<T>` and `ResponseKey` derive macro for predictable API responses.
- Health check endpoint for Postgres.
- Static file and SPA serving helpers.
- Optional Slack/Discord/Sentry error notification integration.