use axum::Json;
use axum::extract::State;
use axum::http::StatusCode;
use serde::Serialize;
use sqlx::PgPool;
use std::time::Duration;
use time::{OffsetDateTime, format_description::well_known::Rfc3339};

#[derive(Serialize)]
pub struct HealthCheck {
    status: String,
    postgres: bool,
    timestamp: String,
}

pub async fn check_health(State(pool): State<PgPool>) -> Result<Json<HealthCheck>, StatusCode> {
    // Try to execute a simple query with timeout
    let db_connected = match tokio::time::timeout(
        Duration::from_secs(5),
        sqlx::query("SELECT (1) as ok").fetch_one(&pool),
    )
    .await
    {
        Ok(Ok(_)) => true,
        Ok(Err(_)) | Err(_) => false,
    };

    let now = OffsetDateTime::now_utc();
    let timestamp = now.format(&Rfc3339).unwrap_or_default();

    let health = HealthCheck {
        status: if db_connected {
            "healthy".to_string()
        } else {
            "degraded".to_string()
        },
        postgres: db_connected,
        timestamp,
    };

    if db_connected {
        Ok(Json(health))
    } else {
        Err(StatusCode::SERVICE_UNAVAILABLE)
    }
}
