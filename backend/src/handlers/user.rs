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
        Err(_) => Err(AppError::user_not_found(&claims.sub)),
    }
}
