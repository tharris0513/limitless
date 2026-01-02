use crate::config_export::GameConfigExport;
use crate::damage_calculator::DamageCalculator;
use crate::error::AppError;
use crate::middleware::AdminClaims;
use crate::models::{
    Ability, Adventure, Character, Class, CreateCreatureRequest, CreateLocationRequest, Creature,
    Location, User,
};
use crate::repository::UserRepository;
use axum::{
    extract::{Path, State},
    Json,
};
use chrono::Utc;
use serde_json::{json, Value};
use std::sync::Arc;
use uuid::Uuid;

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

// ===== LOCATION MANAGEMENT =====

// Admin only - get all locations
pub async fn get_all_locations(
    State(repo): State<Arc<UserRepository>>,
    AdminClaims(claims): AdminClaims,
) -> Result<Json<Vec<Location>>, AppError> {
    tracing::info!("Admin {} requested all locations", claims.sub);

    match repo.get_all_locations().await {
        Ok(locations) => {
            tracing::info!("Returning {} locations to admin", locations.len());
            Ok(Json(locations))
        }
        Err(e) => {
            tracing::error!("Failed to fetch locations: {:?}", e);
            Err(AppError::from(e))
        }
    }
}

// Admin only - create location
pub async fn create_location(
    State(repo): State<Arc<UserRepository>>,
    AdminClaims(_claims): AdminClaims,
    Json(req): Json<CreateLocationRequest>,
) -> Result<Json<Location>, AppError> {
    // Generate new ID and timestamp
    let location_data = Location {
        id: Uuid::new_v4().to_string(),
        name: req.name,
        description: req.description,
        min_level: req.min_level,
        max_level: req.max_level,
        tier: req.tier,
        enabled: req.enabled,
        created_at: Utc::now().to_rfc3339(),
    };

    match repo.create_location(location_data).await {
        Ok(location) => {
            tracing::info!("Created location: {}", location.id);
            Ok(Json(location))
        }
        Err(e) => {
            tracing::error!("Failed to create location: {:?}", e);
            Err(AppError::from(e))
        }
    }
}

// Admin only - update location
pub async fn update_location(
    State(repo): State<Arc<UserRepository>>,
    AdminClaims(_claims): AdminClaims,
    Path(location_id): Path<String>,
    Json(location_data): Json<Location>,
) -> Result<Json<Location>, AppError> {
    match repo.update_location(&location_id, location_data).await {
        Ok(location) => {
            tracing::info!("Updated location: {}", location.id);
            Ok(Json(location))
        }
        Err(e) => {
            tracing::error!("Failed to update location: {:?}", e);
            Err(AppError::from(e))
        }
    }
}

// Admin only - delete location
pub async fn delete_location(
    State(repo): State<Arc<UserRepository>>,
    AdminClaims(_claims): AdminClaims,
    Path(location_id): Path<String>,
) -> Result<Json<Value>, AppError> {
    match repo.delete_location(&location_id).await {
        Ok(()) => Ok(Json(json!({
            "message": "Location deleted successfully",
            "location_id": location_id
        }))),
        Err(e) => {
            tracing::error!("Failed to delete location: {:?}", e);
            Err(AppError::from(e))
        }
    }
}

// Admin only - add creature to location
pub async fn add_creature_to_location(
    State(repo): State<Arc<UserRepository>>,
    AdminClaims(_claims): AdminClaims,
    Path((location_id, creature_id)): Path<(String, String)>,
    Json(payload): Json<Value>,
) -> Result<Json<Value>, AppError> {
    let spawn_rate = payload
        .get("spawnRate")
        .and_then(|v| v.as_i64())
        .ok_or_else(|| AppError::validation_error("Spawn rate is required"))?;

    if !(1..=100).contains(&spawn_rate) {
        return Err(AppError::validation_error(
            "Spawn rate must be between 1 and 100",
        ));
    }

    match repo
        .add_creature_to_location(&location_id, &creature_id, spawn_rate)
        .await
    {
        Ok(()) => Ok(Json(json!({
            "message": "Creature added to location successfully"
        }))),
        Err(e) => {
            tracing::error!("Failed to add creature to location: {:?}", e);
            Err(AppError::from(e))
        }
    }
}

// Admin only - remove creature from location
pub async fn remove_creature_from_location(
    State(repo): State<Arc<UserRepository>>,
    AdminClaims(_claims): AdminClaims,
    Path((location_id, creature_id)): Path<(String, String)>,
) -> Result<Json<Value>, AppError> {
    match repo
        .remove_creature_from_location(&location_id, &creature_id)
        .await
    {
        Ok(()) => Ok(Json(json!({
            "message": "Creature removed from location successfully"
        }))),
        Err(e) => {
            tracing::error!("Failed to remove creature from location: {:?}", e);
            Err(AppError::from(e))
        }
    }
}

// Admin only - add adventure to location
pub async fn add_adventure_to_location(
    State(repo): State<Arc<UserRepository>>,
    AdminClaims(_claims): AdminClaims,
    Path((location_id, adventure_id)): Path<(String, String)>,
    Json(payload): Json<Value>,
) -> Result<Json<Value>, AppError> {
    let spawn_rate = payload
        .get("spawnRate")
        .and_then(|v| v.as_i64())
        .ok_or_else(|| AppError::validation_error("Spawn rate is required"))?;

    if !(1..=100).contains(&spawn_rate) {
        return Err(AppError::validation_error(
            "Spawn rate must be between 1 and 100",
        ));
    }

    match repo
        .add_adventure_to_location(&location_id, &adventure_id, spawn_rate)
        .await
    {
        Ok(()) => Ok(Json(json!({
            "message": "Adventure added to location successfully"
        }))),
        Err(e) => {
            tracing::error!("Failed to add adventure to location: {:?}", e);
            Err(AppError::from(e))
        }
    }
}

// Admin only - remove adventure from location
pub async fn remove_adventure_from_location(
    State(repo): State<Arc<UserRepository>>,
    AdminClaims(_claims): AdminClaims,
    Path((location_id, adventure_id)): Path<(String, String)>,
) -> Result<Json<Value>, AppError> {
    match repo
        .remove_adventure_from_location(&location_id, &adventure_id)
        .await
    {
        Ok(()) => Ok(Json(json!({
            "message": "Adventure removed from location successfully"
        }))),
        Err(e) => {
            tracing::error!("Failed to remove adventure from location: {:?}", e);
            Err(AppError::from(e))
        }
    }
}

// Admin only - get location creatures
pub async fn get_location_creatures(
    State(repo): State<Arc<UserRepository>>,
    AdminClaims(_claims): AdminClaims,
    Path(location_id): Path<String>,
) -> Result<Json<Value>, AppError> {
    match repo.get_location_creatures(&location_id).await {
        Ok(creatures) => Ok(Json(json!({ "creatures": creatures }))),
        Err(e) => {
            tracing::error!("Failed to get location creatures: {:?}", e);
            Err(AppError::from(e))
        }
    }
}

// Admin only - get location adventures
pub async fn get_location_adventures(
    State(repo): State<Arc<UserRepository>>,
    AdminClaims(_claims): AdminClaims,
    Path(location_id): Path<String>,
) -> Result<Json<Value>, AppError> {
    match repo.get_location_adventures(&location_id).await {
        Ok(adventures) => Ok(Json(json!({ "adventures": adventures }))),
        Err(e) => {
            tracing::error!("Failed to get location adventures: {:?}", e);
            Err(AppError::from(e))
        }
    }
}

// ===== CREATURE MANAGEMENT =====

// Admin only - get all creatures
pub async fn get_all_creatures(
    State(repo): State<Arc<UserRepository>>,
    AdminClaims(claims): AdminClaims,
) -> Result<Json<Vec<Creature>>, AppError> {
    tracing::info!("Admin {} requested all creatures", claims.sub);

    match repo.get_all_creatures().await {
        Ok(creatures) => {
            tracing::info!("Returning {} creatures to admin", creatures.len());
            Ok(Json(creatures))
        }
        Err(e) => {
            tracing::error!("Failed to fetch creatures: {:?}", e);
            Err(AppError::from(e))
        }
    }
}

// Admin only - create creature
pub async fn create_creature(
    State(repo): State<Arc<UserRepository>>,
    AdminClaims(_claims): AdminClaims,
    Json(request): Json<CreateCreatureRequest>,
) -> Result<Json<Creature>, AppError> {
    // Build Creature from request and generate id/timestamp
    let creature = Creature {
        id: Uuid::new_v4().to_string(),
        name: request.name,
        description: request.description,
        level: request.level,
        health: request.health,
        might: request.might,
        defense: request.defense,
        magic: request.magic,
        resistance: request.resistance,
        agility: request.agility,
        experience_reward: request.experience_reward,
        creature_type: request.creature_type,
        created_at: Utc::now().to_rfc3339(),
    };

    match repo.create_creature(creature).await {
        Ok(creature) => {
            tracing::info!("Created creature: {}", creature.id);
            Ok(Json(creature))
        }
        Err(e) => {
            tracing::error!("Failed to create creature: {:?}", e);
            Err(AppError::from(e))
        }
    }
}

// Admin only - get creature by ID
pub async fn get_creature(
    State(repo): State<Arc<UserRepository>>,
    AdminClaims(_claims): AdminClaims,
    Path(creature_id): Path<String>,
) -> Result<Json<Creature>, AppError> {
    match repo.get_creature(&creature_id).await {
        Ok(creature) => Ok(Json(creature)),
        Err(e) => {
            tracing::error!("Failed to get creature: {:?}", e);
            Err(AppError::from(e))
        }
    }
}

// Admin only - update creature
pub async fn update_creature(
    State(repo): State<Arc<UserRepository>>,
    AdminClaims(_claims): AdminClaims,
    Path(creature_id): Path<String>,
    Json(creature_data): Json<Creature>,
) -> Result<Json<Creature>, AppError> {
    match repo.update_creature(&creature_id, creature_data).await {
        Ok(creature) => {
            tracing::info!("Updated creature: {}", creature.id);
            Ok(Json(creature))
        }
        Err(e) => {
            tracing::error!("Failed to update creature: {:?}", e);
            Err(AppError::from(e))
        }
    }
}

// Admin only - delete creature
pub async fn delete_creature(
    State(repo): State<Arc<UserRepository>>,
    AdminClaims(_claims): AdminClaims,
    Path(creature_id): Path<String>,
) -> Result<Json<Value>, AppError> {
    match repo.delete_creature(&creature_id).await {
        Ok(()) => Ok(Json(json!({
            "message": "Creature deleted successfully",
            "creature_id": creature_id
        }))),
        Err(e) => {
            tracing::error!("Failed to delete creature: {:?}", e);
            Err(AppError::from(e))
        }
    }
}

// ===== ADVENTURE MANAGEMENT =====

// Admin only - get all adventures
pub async fn get_all_adventures(
    State(repo): State<Arc<UserRepository>>,
    AdminClaims(claims): AdminClaims,
) -> Result<Json<Vec<Adventure>>, AppError> {
    tracing::info!("Admin {} requested all adventures", claims.sub);

    match repo.get_all_adventures().await {
        Ok(adventures) => {
            tracing::info!("Returning {} adventures to admin", adventures.len());
            Ok(Json(adventures))
        }
        Err(e) => {
            tracing::error!("Failed to fetch adventures: {:?}", e);
            Err(AppError::from(e))
        }
    }
}

// Admin only - create adventure
pub async fn create_adventure(
    State(repo): State<Arc<UserRepository>>,
    AdminClaims(_claims): AdminClaims,
    Json(mut adventure_data): Json<Adventure>,
) -> Result<Json<Adventure>, AppError> {
    // Generate new ID and timestamp
    adventure_data.id = Uuid::new_v4().to_string();
    adventure_data.created_at = Utc::now().to_rfc3339();

    match repo.create_adventure(adventure_data).await {
        Ok(adventure) => {
            tracing::info!("Created adventure: {}", adventure.id);
            Ok(Json(adventure))
        }
        Err(e) => {
            tracing::error!("Failed to create adventure: {:?}", e);
            Err(AppError::from(e))
        }
    }
}

// Admin only - update adventure
pub async fn update_adventure(
    State(repo): State<Arc<UserRepository>>,
    AdminClaims(_claims): AdminClaims,
    Path(adventure_id): Path<String>,
    Json(adventure_data): Json<Adventure>,
) -> Result<Json<Adventure>, AppError> {
    match repo.update_adventure(&adventure_id, adventure_data).await {
        Ok(adventure) => {
            tracing::info!("Updated adventure: {}", adventure.id);
            Ok(Json(adventure))
        }
        Err(e) => {
            tracing::error!("Failed to update adventure: {:?}", e);
            Err(AppError::from(e))
        }
    }
}

// Admin only - delete adventure
pub async fn delete_adventure(
    State(repo): State<Arc<UserRepository>>,
    AdminClaims(_claims): AdminClaims,
    Path(adventure_id): Path<String>,
) -> Result<Json<Value>, AppError> {
    match repo.delete_adventure(&adventure_id).await {
        Ok(()) => Ok(Json(json!({
            "message": "Adventure deleted successfully",
            "adventure_id": adventure_id
        }))),
        Err(e) => {
            tracing::error!("Failed to delete adventure: {:?}", e);
            Err(AppError::from(e))
        }
    }
}
