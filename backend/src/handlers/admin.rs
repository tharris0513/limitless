use crate::error::AppError;
use crate::game::abilities::get_ability_by_id;
use crate::handlers::rollover;
use crate::middleware::AdminClaims;
use crate::models::{Character, User};
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

// Admin only - ban user
pub async fn ban_user(
    State(repo): State<Arc<UserRepository>>,
    AdminClaims(_claims): AdminClaims,
    Path(user_id): Path<String>,
) -> Result<Json<User>, AppError> {
    match repo.update_user_ban_status(&user_id, true).await {
        Ok(user) => Ok(Json(user)),
        Err(e) => {
            tracing::error!("Failed to ban user: {:?}", e);
            Err(AppError::from(e))
        }
    }
}

// Admin only - unban user
pub async fn unban_user(
    State(repo): State<Arc<UserRepository>>,
    AdminClaims(_claims): AdminClaims,
    Path(user_id): Path<String>,
) -> Result<Json<User>, AppError> {
    match repo.update_user_ban_status(&user_id, false).await {
        Ok(user) => Ok(Json(user)),
        Err(e) => {
            tracing::error!("Failed to unban user: {:?}", e);
            Err(AppError::from(e))
        }
    }
}

// Admin only - get all characters from all users
pub async fn get_all_characters(
    State(repo): State<Arc<UserRepository>>,
    AdminClaims(claims): AdminClaims,
) -> Result<Json<Vec<Character>>, AppError> {
    tracing::info!("Admin {} requested all characters", claims.sub);

    match repo.get_all_characters().await {
        Ok(characters) => {
            tracing::info!("Returning {} characters to admin", characters.len());
            Ok(Json(characters))
        }
        Err(e) => {
            tracing::error!("Failed to fetch all characters: {:?}", e);
            Err(AppError::from(e))
        }
    }
}

// Admin only - get a single character with abilities
pub async fn get_character_detail(
    State(repo): State<Arc<UserRepository>>,
    AdminClaims(claims): AdminClaims,
    Path(character_id): Path<String>,
) -> Result<Json<Value>, AppError> {
    tracing::info!(
        "Admin {} requested character details for {}",
        claims.sub,
        character_id
    );

    match repo.get_character_by_id(&character_id).await {
        Ok(character) => {
            // Fetch the character's abilities
            let character_abilities = match repo.get_character_abilities(&character_id).await {
                Ok(abs) => abs,
                Err(e) => {
                    tracing::warn!("Failed to fetch character abilities: {:?}", e);
                    Vec::new()
                }
            };

            // Fetch full ability data for each character ability
            let mut abilities = Vec::new();
            for char_ability in character_abilities {
                if let Ok(ability) = get_ability_by_id(&char_ability.ability_id) {
                    abilities.push(ability);
                }
            }

            let response = json!({
                "id": character.id,
                "userId": character.user_id,
                "name": character.name,
                "classId": character.class_id,
                "level": character.level,
                "experience": character.experience,
                "experienceToNext": character.experience_to_next,
                "health": character.health,
                "maxHealth": character.max_health,
                "mana": character.mana,
                "maxMana": character.max_mana,
                "might": character.might,
                "defense": character.defense,
                "magic": character.magic,
                "resistance": character.resistance,
                "agility": character.agility,
                "adventures": character.adventures,
                "location": character.location,
                "gameState": character.game_state,
                "createdAt": character.created_at,
                "lastPlayed": character.last_played,
                "abilities": abilities
            });

            Ok(Json(response))
        }
        Err(e) => {
            tracing::error!("Failed to fetch character detail: {:?}", e);
            Err(AppError::from(e))
        }
    }
}

// Admin only - update character name
pub async fn update_character_name(
    State(repo): State<Arc<UserRepository>>,
    AdminClaims(_claims): AdminClaims,
    Path(character_id): Path<String>,
    Json(payload): Json<Value>,
) -> Result<Json<Character>, AppError> {
    let name = payload
        .get("name")
        .and_then(|v| v.as_str())
        .ok_or_else(|| AppError::validation_error("Character name is required"))?;

    // Validate name
    if name.trim().is_empty() {
        return Err(AppError::validation_error("Character name cannot be empty"));
    }
    if name.len() < 2 || name.len() > 30 {
        return Err(AppError::validation_error(
            "Character name must be between 2 and 30 characters",
        ));
    }

    match repo.update_character_name(&character_id, name).await {
        Ok(character) => Ok(Json(character)),
        Err(e) => {
            tracing::error!("Failed to update character name: {:?}", e);
            Err(AppError::from(e))
        }
    }
}

// Admin only - update character adventures
pub async fn update_character_adventures(
    State(repo): State<Arc<UserRepository>>,
    AdminClaims(_claims): AdminClaims,
    Path(character_id): Path<String>,
    Json(payload): Json<Value>,
) -> Result<Json<Character>, AppError> {
    let adventures = payload
        .get("adventures")
        .and_then(|v| v.as_i64())
        .ok_or_else(|| AppError::validation_error("Adventures count is required"))?;

    // Validate adventures count
    if adventures < 0 {
        return Err(AppError::validation_error(
            "Adventures count cannot be negative",
        ));
    }

    match repo
        .update_character_adventures(&character_id, adventures)
        .await
    {
        Ok(character) => Ok(Json(character)),
        Err(e) => {
            tracing::error!("Failed to update character adventures: {:?}", e);
            Err(AppError::from(e))
        }
    }
}

// Admin only - delete character
pub async fn delete_character(
    State(repo): State<Arc<UserRepository>>,
    AdminClaims(_claims): AdminClaims,
    Path(character_id): Path<String>,
) -> Result<Json<Value>, AppError> {
    match repo.delete_character(&character_id).await {
        Ok(()) => Ok(Json(json!({
            "message": "Character deleted successfully",
            "character_id": character_id
        }))),
        Err(e) => {
            tracing::error!("Failed to delete character: {:?}", e);
            Err(AppError::from(e))
        }
    }
}

// Admin only - get maintenance mode status
pub async fn get_maintenance_mode(
    State(repo): State<Arc<UserRepository>>,
    AdminClaims(_claims): AdminClaims,
) -> Result<Json<Value>, AppError> {
    match repo.get_maintenance_mode().await {
        Ok(enabled) => Ok(Json(json!({
            "enabled": enabled
        }))),
        Err(e) => {
            tracing::error!("Failed to get maintenance mode: {:?}", e);
            Err(AppError::from(e))
        }
    }
}

// Admin only - set maintenance mode status
pub async fn set_maintenance_mode(
    State(repo): State<Arc<UserRepository>>,
    AdminClaims(claims): AdminClaims,
    Json(payload): Json<Value>,
) -> Result<Json<Value>, AppError> {
    let enabled = payload
        .get("enabled")
        .and_then(|v| v.as_bool())
        .ok_or_else(|| AppError::validation_error("enabled field is required"))?;

    match repo.set_maintenance_mode(enabled).await {
        Ok(()) => {
            tracing::info!("Admin {} set maintenance mode to {}", claims.sub, enabled);
            Ok(Json(json!({
                "message": "Maintenance mode updated",
                "enabled": enabled
            })))
        }
        Err(e) => {
            tracing::error!("Failed to set maintenance mode: {:?}", e);
            Err(AppError::from(e))
        }
    }
}

// Admin only - manually trigger rollover
pub async fn trigger_rollover(
    State(repo): State<Arc<UserRepository>>,
    AdminClaims(claims): AdminClaims,
) -> Result<Json<Value>, AppError> {
    tracing::info!("Admin {} manually triggered rollover", claims.sub);

    // Spawn rollover in background to avoid timeout
    let repo_clone = repo.clone();
    tokio::spawn(async move {
        rollover::perform_rollover(repo_clone).await;
    });

    Ok(Json(json!({
        "message": "Rollover triggered successfully. Check server logs for progress.",
        "triggered_by": claims.sub
    })))
}
