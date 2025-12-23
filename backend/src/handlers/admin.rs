use crate::error::AppError;
use crate::middleware::AdminClaims;
use crate::models::User;
use crate::repository::UserRepository;
use axum::{extract::State, Json};
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
