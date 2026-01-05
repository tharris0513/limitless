use crate::error::AppError;
use crate::middleware::AuthClaims;
use crate::models::Character;
use crate::repository::UserRepository;
use axum::{
    extract::{Path, State},
    Json,
};
use std::sync::Arc;

// Get all characters for the current user
pub async fn get_user_characters(
    State(repo): State<Arc<UserRepository>>,
    AuthClaims(claims): AuthClaims,
) -> Result<Json<Vec<Character>>, AppError> {
    match repo.get_user_characters(&claims.sub).await {
        Ok(characters) => Ok(Json(characters)),
        Err(e) => Err(AppError::from(e)),
    }
}

// Create a new character for the current user
pub async fn create_character(
    State(repo): State<Arc<UserRepository>>,
    AuthClaims(claims): AuthClaims,
    Json(payload): Json<serde_json::Value>,
) -> Result<Json<Character>, AppError> {
    let name = payload
        .get("name")
        .and_then(|n| n.as_str())
        .ok_or_else(|| AppError::validation_error("Character name is required"))?;

    let class_id = payload
        .get("classId")
        .and_then(|c| c.as_str())
        .ok_or_else(|| AppError::validation_error("Class ID is required"))?;

    match repo.create_character(&claims.sub, name, class_id).await {
        Ok(character) => Ok(Json(character)),
        Err(e) => {
            tracing::error!("Failed to create character: {:?}", e);
            Err(AppError::from(e))
        }
    }
}

// Get all available classes
pub async fn get_classes(
    State(repo): State<Arc<UserRepository>>,
) -> Result<Json<Vec<crate::models::Class>>, AppError> {
    match repo.get_all_classes().await {
        Ok(classes) => Ok(Json(classes)),
        Err(e) => {
            tracing::error!("Failed to get classes: {:?}", e);
            Err(AppError::from(e))
        }
    }
}

// Get abilities for a class
pub async fn get_class_abilities(
    State(repo): State<Arc<UserRepository>>,
    Path(class_id): Path<String>,
) -> Result<Json<Vec<(crate::models::Ability, i64)>>, AppError> {
    match repo.get_class_abilities(&class_id).await {
        Ok(abilities) => Ok(Json(abilities)),
        Err(e) => {
            tracing::error!("Failed to get class abilities: {:?}", e);
            Err(AppError::from(e))
        }
    }
}

// Get abilities for a character
pub async fn get_character_abilities(
    State(repo): State<Arc<UserRepository>>,
    Path(character_id): Path<String>,
    AuthClaims(_claims): AuthClaims,
) -> Result<Json<Vec<crate::models::CharacterAbility>>, AppError> {
    match repo.get_character_abilities(&character_id).await {
        Ok(abilities) => Ok(Json(abilities)),
        Err(e) => {
            tracing::error!("Failed to get character abilities: {:?}", e);
            Err(AppError::from(e))
        }
    }
}

// Update character
pub async fn update_character(
    State(_repo): State<Arc<UserRepository>>,
    Path(_character_id): Path<String>,
    AuthClaims(_claims): AuthClaims,
) -> Result<Json<Character>, AppError> {
    // TODO: Implement character updates in repository
    // For now, return error as this is not implemented yet
    Err(AppError::BadRequest(
        "Character updates not yet implemented".to_string(),
    ))
}

// Update character's last played timestamp
pub async fn update_character_last_played(
    State(repo): State<Arc<UserRepository>>,
    Path(character_id): Path<String>,
    AuthClaims(_claims): AuthClaims,
) -> Result<Json<serde_json::Value>, AppError> {
    // TODO: Verify character belongs to authenticated user
    match repo.update_character_last_played(&character_id).await {
        Ok(()) => {
            tracing::info!("Updated last played for character: {}", character_id);
            Ok(Json(serde_json::json!({
                "message": "Character last played updated",
                "timestamp": chrono::Utc::now().to_rfc3339()
            })))
        }
        Err(e) => {
            tracing::error!("Failed to update character last played: {:?}", e);
            Err(AppError::from(e))
        }
    }
}
