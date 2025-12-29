use crate::error::AppError;
use crate::middleware::AdminClaims;
use crate::models::User;
use crate::repository::UserRepository;
use axum::{
    extract::{Path, State},
    Json,
};
use serde_json::{json, Value};
use std::sync::Arc;

// Admin only - get all users from database
pub async fn get_all_users(
    State(repo): State<Arc<UserRepository>>,
    AdminClaims(claims): AdminClaims, // AdminClaims extractor enforces admin-only access
) -> Result<Json<Vec<User>>, AppError> {
    // AdminClaims extractor has already verified:
    // 1. User is authenticated (valid JWT)
    // 2. User has admin: true in the database
    // Only admins can reach this point

    tracing::info!("Admin {} requested user list", claims.sub);

    match repo.get_all_users().await {
        Ok(users) => {
            tracing::info!("Returning {} users to admin", users.len());
            Ok(Json(users))
        }
        Err(e) => {
            tracing::error!("Failed to fetch users: {:?}", e);
            Err(AppError::from(e))
        }
    }
}

// Check if current user is admin (useful for frontend)
pub async fn check_admin_status(AdminClaims(claims): AdminClaims) -> Result<Json<Value>, AppError> {
    Ok(Json(json!({
        "is_admin": true,
        "user_id": claims.sub
    })))
}

// Admin only - update user's username
pub async fn update_user_username(
    State(repo): State<Arc<UserRepository>>,
    AdminClaims(_claims): AdminClaims,
    Path(user_id): Path<String>,
    Json(payload): Json<Value>,
) -> Result<Json<User>, AppError> {
    let username = payload
        .get("username")
        .and_then(|v| v.as_str())
        .ok_or_else(|| AppError::validation_error("Username is required"))?;

    // Validate username
    if username.trim().is_empty() {
        return Err(AppError::validation_error("Username cannot be empty"));
    }
    if username.len() < 3 || username.len() > 20 {
        return Err(AppError::validation_error(
            "Username must be between 3 and 20 characters",
        ));
    }

    match repo.update_user(&user_id, Some(username), None).await {
        Ok(user) => Ok(Json(user)),
        Err(e) => {
            tracing::error!("Failed to update username: {:?}", e);
            Err(AppError::from(e))
        }
    }
}

// Admin only - delete user
pub async fn delete_user(
    State(repo): State<Arc<UserRepository>>,
    AdminClaims(_claims): AdminClaims,
    Path(user_id): Path<String>,
) -> Result<Json<Value>, AppError> {
    match repo.delete_user(&user_id).await {
        Ok(()) => Ok(Json(json!({
            "message": "User deleted successfully",
            "user_id": user_id
        }))),
        Err(e) => {
            tracing::error!("Failed to delete user: {:?}", e);
            Err(AppError::from(e))
        }
    }
}
