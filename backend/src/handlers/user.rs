use crate::error::AppError;
use crate::middleware::AuthClaims;
use crate::models::User;
use crate::repository::UserRepository;
use axum::{extract::State, Json};
use serde::Serialize;
use std::sync::Arc;

#[derive(Serialize)]
pub struct UsernameCheckResponse {
    available: bool,
}

// Check if a username is available (no auth required)
pub async fn check_username_available(
    State(repo): State<Arc<UserRepository>>,
    Json(payload): Json<serde_json::Value>,
) -> Result<Json<UsernameCheckResponse>, AppError> {
    let username = payload
        .get("username")
        .and_then(|v| v.as_str())
        .ok_or_else(|| AppError::validation_error("Username is required"))?;

    // Basic validation
    if username.trim().is_empty() || username.len() < 3 {
        return Ok(Json(UsernameCheckResponse { available: false }));
    }

    // Check if username exists
    let all_users = repo.get_all_users().await?;
    let username_taken = all_users
        .iter()
        .any(|u| u.username.as_ref().map(|un| un.to_lowercase()) == Some(username.to_lowercase()));

    Ok(Json(UsernameCheckResponse {
        available: !username_taken,
    }))
}

// Get current user info
pub async fn get_user(
    State(repo): State<Arc<UserRepository>>,
    AuthClaims(claims): AuthClaims,
) -> Result<Json<User>, AppError> {
    match repo.find_by_id(&claims.sub).await {
        Ok(user) => Ok(Json(user)),
        Err(_) => {
            // User doesn't exist in database (e.g., after migration to DynamoDB)
            // Return 401 to force re-login via OAuth
            Err(AppError::authentication_error(
                "User session expired. Please log in again.",
            ))
        }
    }
}

// Update user info (username, date of birth, etc.)
pub async fn update_user(
    State(repo): State<Arc<UserRepository>>,
    AuthClaims(claims): AuthClaims,
    Json(payload): Json<serde_json::Value>,
) -> Result<Json<User>, AppError> {
    // Extract optional fields from payload
    let username = payload.get("username").and_then(|v| v.as_str());
    let date_of_birth = payload.get("dateOfBirth").and_then(|v| v.as_str());

    // Validate username if provided
    if let Some(username) = username {
        if username.trim().is_empty() {
            return Err(AppError::validation_error("Username cannot be empty"));
        }
        if username.len() < 3 || username.len() > 20 {
            return Err(AppError::validation_error(
                "Username must be between 3 and 20 characters",
            ));
        }
        if !username
            .chars()
            .all(|c| c.is_alphanumeric() || c == '_' || c == '-')
        {
            return Err(AppError::validation_error(
                "Username can only contain letters, numbers, underscores, and hyphens",
            ));
        }
    }

    // Validate date of birth if provided
    if let Some(dob) = date_of_birth {
        // Basic ISO date format validation
        if chrono::NaiveDate::parse_from_str(dob, "%Y-%m-%d").is_err() {
            return Err(AppError::validation_error(
                "Invalid date of birth format. Use YYYY-MM-DD",
            ));
        }
    }

    match repo.update_user(&claims.sub, username, date_of_birth).await {
        Ok(user) => Ok(Json(user)),
        Err(e) => {
            tracing::error!("Failed to update user: {:?}", e);
            Err(AppError::from(e))
        }
    }
}
