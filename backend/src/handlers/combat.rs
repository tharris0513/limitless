use crate::error::AppError;
use crate::middleware::AuthClaims;
use crate::models::Character;
use crate::repository::UserRepository;
use axum::{
    extract::{Path, State},
    Json,
};
use std::sync::Arc;

// Save character's game state
pub async fn save_game_state(
    State(repo): State<Arc<UserRepository>>,
    Path(character_id): Path<String>,
    AuthClaims(claims): AuthClaims,
    Json(game_state): Json<serde_json::Value>,
) -> Result<Json<serde_json::Value>, AppError> {
    // Convert game state to JSON string
    let game_state_str = serde_json::to_string(&game_state)
        .map_err(|e| AppError::validation_error(&format!("Invalid game state JSON: {}", e)))?;

    match repo
        .update_character_game_state(&character_id, &claims.sub, Some(game_state_str))
        .await
    {
        Ok(()) => {
            tracing::info!("Saved game state for character: {}", character_id);
            Ok(Json(serde_json::json!({
                "message": "Game state saved",
                "timestamp": chrono::Utc::now().to_rfc3339()
            })))
        }
        Err(e) => {
            tracing::error!("Failed to save game state: {:?}", e);
            Err(AppError::from(e))
        }
    }
}

// Clear character's game state (return to idle)
pub async fn clear_game_state(
    State(repo): State<Arc<UserRepository>>,
    Path(character_id): Path<String>,
    AuthClaims(claims): AuthClaims,
) -> Result<Json<serde_json::Value>, AppError> {
    match repo
        .update_character_game_state(&character_id, &claims.sub, None)
        .await
    {
        Ok(()) => {
            tracing::info!("Cleared game state for character: {}", character_id);
            Ok(Json(serde_json::json!({
                "message": "Game state cleared",
                "timestamp": chrono::Utc::now().to_rfc3339()
            })))
        }
        Err(e) => {
            tracing::error!("Failed to clear game state: {:?}", e);
            Err(AppError::from(e))
        }
    }
}

// Flee from combat - clear game state and consume 1 adventure
pub async fn flee_combat(
    State(repo): State<Arc<UserRepository>>,
    Path(character_id): Path<String>,
    AuthClaims(claims): AuthClaims,
) -> Result<Json<Character>, AppError> {
    // Get all user characters to verify ownership and get current adventures
    let characters = repo
        .get_user_characters(&claims.sub)
        .await
        .map_err(|e| AppError::from(e))?;

    let character = characters
        .iter()
        .find(|c| c.id == character_id)
        .ok_or_else(|| AppError::character_not_found(&character_id))?;

    // Check if character has adventures available
    if character.stats.adventures <= 0 {
        return Err(AppError::validation_error("No adventures remaining"));
    }

    // Decrease adventures by 1
    let new_adventures = character.stats.adventures - 1;

    // Update character adventures
    let updated_character = repo
        .update_character_adventures(&character_id, new_adventures)
        .await
        .map_err(|e| {
            tracing::error!("Failed to update adventures: {:?}", e);
            AppError::from(e)
        })?;

    // Clear game state
    repo.update_character_game_state(&character_id, &claims.sub, None)
        .await
        .map_err(|e| {
            tracing::error!("Failed to clear game state: {:?}", e);
            AppError::from(e)
        })?;

    tracing::info!("Character {} fled from combat", character_id);
    Ok(Json(updated_character))
}
