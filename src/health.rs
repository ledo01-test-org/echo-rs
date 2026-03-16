use axum::{http::StatusCode, response::IntoResponse, Json};
use serde::Serialize;

#[derive(Serialize)]
pub struct HealthResponse {
    pub status: &'static str,
    pub version: &'static str,
}

#[derive(Serialize)]
pub struct ReadinessResponse {
    pub ready: bool,
    pub checks: ReadinessChecks,
}

#[derive(Serialize)]
pub struct ReadinessChecks {
    pub server: bool,
    pub broadcast_channel: bool,
}

pub async fn health_check() -> impl IntoResponse {
    (
        StatusCode::OK,
        Json(HealthResponse {
            status: "healthy",
            version: env!("CARGO_PKG_VERSION"),
        }),
    )
}

pub async fn readiness_check() -> impl IntoResponse {
    let response = ReadinessResponse {
        ready: true,
        checks: ReadinessChecks {
            server: true,
            broadcast_channel: true,
        },
    };

    (StatusCode::OK, Json(response))
}
