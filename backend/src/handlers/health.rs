use crate::jwt::JwtService;
use crate::models::Claims;
use crate::repository::UserRepository;
use axum::{extract::State, http::StatusCode, Json};
use serde::Serialize;
use std::{env, sync::Arc};

// Health check models
#[derive(Serialize)]
pub struct HealthStatus {
    status: String,
    timestamp: String,
    version: String,
    checks: HealthChecks,
}

#[derive(Serialize)]
pub struct HealthChecks {
    database: ComponentHealth,
    auth: ComponentHealth,
}

#[derive(Serialize)]
pub struct ComponentHealth {
    status: String,
    response_time_ms: u64,
    details: Option<String>,
}

// Health check endpoint
pub async fn health_check(
    State(repo): State<Arc<UserRepository>>,
) -> Result<Json<HealthStatus>, StatusCode> {
    // Check database connectivity
    let db_start = std::time::Instant::now();
    let db_status = match repo.health_check().await {
        Ok(_) => ComponentHealth {
            status: "healthy".to_string(),
            response_time_ms: db_start.elapsed().as_millis() as u64,
            details: Some("Database connection successful".to_string()),
        },
        Err(e) => ComponentHealth {
            status: "unhealthy".to_string(),
            response_time_ms: db_start.elapsed().as_millis() as u64,
            details: Some(format!("Database error: {}", e)),
        },
    };

    // Check auth system (JWT generation/verification)
    let auth_start = std::time::Instant::now();
    let test_claims = Claims {
        sub: "health-check".to_string(),
        discord_id: "test".to_string(),
        admin: false,
        exp: (chrono::Utc::now() + chrono::Duration::hours(1)).timestamp() as usize,
    };
    
    let auth_status = match JwtService::generate_token_for_claims(&test_claims) {
        Ok(token) => match JwtService::verify_token(&token) {
            Ok(_) => ComponentHealth {
                status: "healthy".to_string(),
                response_time_ms: auth_start.elapsed().as_millis() as u64,
                details: Some("JWT generation and verification successful".to_string()),
            },
            Err(e) => ComponentHealth {
                status: "unhealthy".to_string(),
                response_time_ms: auth_start.elapsed().as_millis() as u64,
                details: Some(format!("JWT verification failed: {}", e)),
            },
        },
        Err(e) => ComponentHealth {
            status: "unhealthy".to_string(),
            response_time_ms: auth_start.elapsed().as_millis() as u64,
            details: Some(format!("JWT generation failed: {}", e)),
        },
    };

    // Overall health status
    let overall_status = if db_status.status == "healthy" && auth_status.status == "healthy" {
        "healthy"
    } else {
        "unhealthy"
    };

    let health_status = HealthStatus {
        status: overall_status.to_string(),
        timestamp: chrono::Utc::now().to_rfc3339(),
        version: env!("CARGO_PKG_VERSION").to_string(),
        checks: HealthChecks {
            database: db_status,
            auth: auth_status,
        },
    };

    // Return appropriate HTTP status
    match overall_status {
        "healthy" => Ok(Json(health_status)),
        _ => Err(StatusCode::SERVICE_UNAVAILABLE),
    }
}