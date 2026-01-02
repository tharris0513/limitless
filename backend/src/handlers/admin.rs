use crate::config_export::GameConfigExport;
use crate::damage_calculator::DamageCalculator;
use crate::error::AppError;
use crate::middleware::AdminClaims;
use crate::models::{Ability, Character, Class, User};
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

// Admin only - create a new class
pub async fn create_class(
    State(repo): State<Arc<UserRepository>>,
    AdminClaims(_claims): AdminClaims,
    Json(class_data): Json<Class>,
) -> Result<Json<Class>, AppError> {
    match repo.create_class(class_data).await {
        Ok(class) => Ok(Json(class)),
        Err(e) => {
            tracing::error!("Failed to create class: {:?}", e);
            Err(AppError::from(e))
        }
    }
}

// Admin only - update a class
pub async fn update_class(
    State(repo): State<Arc<UserRepository>>,
    AdminClaims(_claims): AdminClaims,
    Path(class_id): Path<String>,
    Json(class_data): Json<Class>,
) -> Result<Json<Class>, AppError> {
    match repo.update_class(&class_id, class_data).await {
        Ok(class) => Ok(Json(class)),
        Err(e) => {
            tracing::error!("Failed to update class: {:?}", e);
            Err(AppError::from(e))
        }
    }
}

// Admin only - delete a class
pub async fn delete_class(
    State(repo): State<Arc<UserRepository>>,
    AdminClaims(_claims): AdminClaims,
    Path(class_id): Path<String>,
) -> Result<Json<Value>, AppError> {
    match repo.delete_class(&class_id).await {
        Ok(()) => Ok(Json(json!({
            "message": "Class deleted successfully",
            "class_id": class_id
        }))),
        Err(e) => {
            tracing::error!("Failed to delete class: {:?}", e);
            Err(AppError::from(e))
        }
    }
}

// Admin only - add ability to class
pub async fn add_class_ability(
    State(repo): State<Arc<UserRepository>>,
    AdminClaims(_claims): AdminClaims,
    Path(class_id): Path<String>,
    Json(payload): Json<Value>,
) -> Result<Json<Value>, AppError> {
    let ability_id = payload
        .get("abilityId")
        .and_then(|v| v.as_str())
        .ok_or_else(|| AppError::validation_error("Ability ID is required"))?;

    let unlock_level = payload
        .get("unlockLevel")
        .and_then(|v| v.as_i64())
        .ok_or_else(|| AppError::validation_error("Unlock level is required"))?;

    match repo
        .add_class_ability(&class_id, ability_id, unlock_level)
        .await
    {
        Ok(()) => Ok(Json(json!({
            "message": "Ability added to class",
            "class_id": class_id,
            "ability_id": ability_id,
            "unlock_level": unlock_level
        }))),
        Err(e) => {
            tracing::error!("Failed to add ability to class: {:?}", e);
            Err(AppError::from(e))
        }
    }
}

// Admin only - remove ability from class
pub async fn remove_class_ability(
    State(repo): State<Arc<UserRepository>>,
    AdminClaims(_claims): AdminClaims,
    Path((class_id, ability_id)): Path<(String, String)>,
) -> Result<Json<Value>, AppError> {
    match repo.remove_class_ability(&class_id, &ability_id).await {
        Ok(()) => Ok(Json(json!({
            "message": "Ability removed from class",
            "class_id": class_id,
            "ability_id": ability_id
        }))),
        Err(e) => {
            tracing::error!("Failed to remove ability from class: {:?}", e);
            Err(AppError::from(e))
        }
    }
}

// Admin only - create a new ability
pub async fn create_ability(
    State(repo): State<Arc<UserRepository>>,
    AdminClaims(_claims): AdminClaims,
    Json(ability_data): Json<Ability>,
) -> Result<Json<Ability>, AppError> {
    // Validate formulas if present
    if let Some(ref formula) = ability_data.damage_formula {
        DamageCalculator::validate_formula(formula).map_err(|e| AppError::BadRequest(e))?;
    }
    if let Some(ref formula) = ability_data.heal_formula {
        DamageCalculator::validate_formula(formula).map_err(|e| AppError::BadRequest(e))?;
    }
    if let Some(ref formula) = ability_data.effect_formula {
        DamageCalculator::validate_formula(formula).map_err(|e| AppError::BadRequest(e))?;
    }

    match repo.create_ability(ability_data).await {
        Ok(ability) => Ok(Json(ability)),
        Err(e) => {
            tracing::error!("Failed to create ability: {:?}", e);
            Err(AppError::from(e))
        }
    }
}

// Admin only - update an ability
pub async fn update_ability(
    State(repo): State<Arc<UserRepository>>,
    AdminClaims(_claims): AdminClaims,
    Path(ability_id): Path<String>,
    Json(ability_data): Json<Ability>,
) -> Result<Json<Ability>, AppError> {
    // Validate formulas if present
    if let Some(ref formula) = ability_data.damage_formula {
        DamageCalculator::validate_formula(formula).map_err(|e| AppError::BadRequest(e))?;
    }
    if let Some(ref formula) = ability_data.heal_formula {
        DamageCalculator::validate_formula(formula).map_err(|e| AppError::BadRequest(e))?;
    }
    if let Some(ref formula) = ability_data.effect_formula {
        DamageCalculator::validate_formula(formula).map_err(|e| AppError::BadRequest(e))?;
    }

    match repo.update_ability(&ability_id, ability_data).await {
        Ok(ability) => Ok(Json(ability)),
        Err(e) => {
            tracing::error!("Failed to update ability: {:?}", e);
            Err(AppError::from(e))
        }
    }
}

// Admin only - delete an ability
pub async fn delete_ability(
    State(repo): State<Arc<UserRepository>>,
    AdminClaims(_claims): AdminClaims,
    Path(ability_id): Path<String>,
) -> Result<Json<Value>, AppError> {
    match repo.delete_ability(&ability_id).await {
        Ok(()) => Ok(Json(json!({
            "message": "Ability deleted successfully",
            "ability_id": ability_id
        }))),
        Err(e) => {
            tracing::error!("Failed to delete ability: {:?}", e);
            Err(AppError::from(e))
        }
    }
}

// Admin only - get all abilities
pub async fn get_all_abilities(
    State(repo): State<Arc<UserRepository>>,
    AdminClaims(_claims): AdminClaims,
) -> Result<Json<Vec<Ability>>, AppError> {
    match repo.get_all_abilities().await {
        Ok(abilities) => Ok(Json(abilities)),
        Err(e) => {
            tracing::error!("Failed to get abilities: {:?}", e);
            Err(AppError::from(e))
        }
    }
}

// Admin only - export game configuration (classes and abilities)
pub async fn export_game_config(
    State(repo): State<Arc<UserRepository>>,
    AdminClaims(claims): AdminClaims,
) -> Result<Json<GameConfigExport>, AppError> {
    tracing::info!("Admin {} exporting game configuration", claims.sub);

    match repo.export_game_config().await {
        Ok(config) => {
            tracing::info!(
                "Exported {} classes and {} abilities",
                config.classes.len(),
                config.abilities.len()
            );
            Ok(Json(config))
        }
        Err(e) => {
            tracing::error!("Failed to export game config: {:?}", e);
            Err(AppError::from(e))
        }
    }
}

// Admin only - import game configuration (classes and abilities)
pub async fn import_game_config(
    State(repo): State<Arc<UserRepository>>,
    AdminClaims(claims): AdminClaims,
    Json(config): Json<GameConfigExport>,
) -> Result<Json<Value>, AppError> {
    tracing::info!(
        "Admin {} importing game configuration: {} classes, {} abilities",
        claims.sub,
        config.classes.len(),
        config.abilities.len()
    );

    match repo.import_game_config(config).await {
        Ok(()) => {
            tracing::info!("Successfully imported game configuration");
            Ok(Json(json!({
                "message": "Game configuration imported successfully"
            })))
        }
        Err(e) => {
            tracing::error!("Failed to import game config: {:?}", e);
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
        return Err(AppError::validation_error("Adventures count cannot be negative"));
    }

    match repo.update_character_adventures(&character_id, adventures).await {
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
