use crate::error::{ApiError, AppError};
use crate::jwt::JwtService;
use crate::models::Claims;
use axum::{
    extract::{FromRequestParts, Request},
    http::{request::Parts, HeaderMap, StatusCode},
    middleware::Next,
    response::{IntoResponse, Response},
    Json,
};

// Extension to add claims to request
#[derive(Clone)]
pub struct AuthClaims(pub Claims);

// Admin-only claims - requires user to be an admin
#[derive(Clone)]
pub struct AdminClaims(pub Claims);

// Implement FromRequestParts to make AuthClaims an extractor
#[axum::async_trait]
impl<S> FromRequestParts<S> for AuthClaims
where
    S: Send + Sync,
{
    type Rejection = AppError;

    async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
        parts
            .extensions
            .get::<AuthClaims>()
            .cloned()
            .ok_or_else(|| AppError::authentication_error("Authentication required"))
    }
}

// Implement FromRequestParts for AdminClaims - requires admin privileges
#[axum::async_trait]
impl<S> FromRequestParts<S> for AdminClaims
where
    S: Send + Sync,
{
    type Rejection = AppError;

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        // First extract auth claims
        let AuthClaims(claims) = AuthClaims::from_request_parts(parts, state).await?;

        // Check if user is admin
        if !claims.admin {
            return Err(AppError::forbidden());
        }

        Ok(AdminClaims(claims))
    }
}

pub async fn jwt_auth_middleware(
    headers: HeaderMap,
    mut request: Request,
    next: Next,
) -> Result<Response, AppError> {
    let auth_header = headers
        .get("authorization")
        .and_then(|header| header.to_str().ok());

    if let Some(auth_header) = auth_header {
        if let Some(token) = JwtService::extract_token_from_auth_header(auth_header) {
            match JwtService::verify_token(token) {
                Ok(claims) => {
                    // Add claims to request extensions
                    request.extensions_mut().insert(AuthClaims(claims));
                    return Ok(next.run(request).await);
                }
                Err(_) => {
                    return Err(AppError::InvalidToken);
                }
            }
        }
    }

    Err(AppError::authentication_error(
        "Authentication token required",
    ))
}

// Global error handler middleware
pub async fn global_error_handler(request: Request, next: Next) -> Response {
    let response = next.run(request).await;

    // If the response is an error status, try to extract and format it
    if response.status().is_client_error() || response.status().is_server_error() {
        let status = response.status();

        // Create a standardized error response for unhandled errors
        let api_error = ApiError {
            error: match status {
                StatusCode::NOT_FOUND => "NOT_FOUND".to_string(),
                StatusCode::UNAUTHORIZED => "UNAUTHORIZED".to_string(),
                StatusCode::FORBIDDEN => "FORBIDDEN".to_string(),
                StatusCode::BAD_REQUEST => "BAD_REQUEST".to_string(),
                StatusCode::INTERNAL_SERVER_ERROR => "INTERNAL_SERVER_ERROR".to_string(),
                StatusCode::SERVICE_UNAVAILABLE => "SERVICE_UNAVAILABLE".to_string(),
                _ => "UNKNOWN_ERROR".to_string(),
            },
            message: match status {
                StatusCode::NOT_FOUND => "Resource not found".to_string(),
                StatusCode::UNAUTHORIZED => "Authentication required".to_string(),
                StatusCode::FORBIDDEN => "Access denied".to_string(),
                StatusCode::BAD_REQUEST => "Invalid request".to_string(),
                StatusCode::INTERNAL_SERVER_ERROR => "Internal server error".to_string(),
                StatusCode::SERVICE_UNAVAILABLE => "Service temporarily unavailable".to_string(),
                _ => "An error occurred".to_string(),
            },
            status_code: status.as_u16(),
            timestamp: chrono::Utc::now().to_rfc3339(),
            details: None,
        };

        // Log the error
        tracing::warn!("Unhandled HTTP error: {} - {}", status, api_error.message);

        // Return the formatted error response
        (status, Json(api_error)).into_response()
    } else {
        response
    }
}
