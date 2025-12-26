use crate::error::AppError;
use crate::middleware::AuthClaims;
use crate::models::User;
use crate::repository::UserRepository;
use axum::{extract::State, Json};
use std::sync::Arc;

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
                "User session expired. Please log in again."
            ))
        }
    }
}
