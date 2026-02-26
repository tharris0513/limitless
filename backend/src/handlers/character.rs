use crate::game::abilities::{get_ability_by_id, AbilityType, ActiveBuff, CharacterAbility};
use crate::game::classes::{get_all_classes, Class};
use crate::middleware::AuthClaims;
use crate::models::Character;
use crate::repository::UserRepository;
use crate::{error::AppError, game::abilities::Ability};
use axum::{
    extract::{Path, State},
    Json,
};
use serde::{Deserialize, Serialize};
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

// Get a single character by ID
pub async fn get_character(
    State(repo): State<Arc<UserRepository>>,
    Path(character_id): Path<String>,
    AuthClaims(claims): AuthClaims,
) -> Result<Json<Character>, AppError> {
    match repo.get_character(&character_id, &claims.sub).await {
        Ok(character) => Ok(Json(character)),
        Err(_) => Err(AppError::character_not_found(&character_id)),
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
pub async fn get_classes() -> Result<Json<Vec<Class>>, AppError> {
    match get_all_classes() {
        Ok(classes) => Ok(Json(classes)),
        Err(e) => {
            tracing::error!("Failed to get classes: {:?}", e);
            Err(AppError::from(e))
        }
    }
}

// Get abilities for a character
pub async fn get_character_abilities(
    State(repo): State<Arc<UserRepository>>,
    Path(character_id): Path<String>,
    AuthClaims(_claims): AuthClaims,
) -> Result<Json<Vec<CharacterAbility>>, AppError> {
    match repo.get_character_abilities(&character_id).await {
        Ok(abilities) => Ok(Json(abilities)),
        Err(e) => {
            tracing::error!("Failed to get character abilities: {:?}", e);
            Err(AppError::from(e))
        }
    }
}

// Get unlocked abilities for a character (full Ability data)
pub async fn get_character_unlocked_abilities(
    State(repo): State<Arc<UserRepository>>,
    Path(character_id): Path<String>,
    AuthClaims(claims): AuthClaims,
) -> Result<Json<Vec<(Ability, i64)>>, AppError> {
    // Verify character belongs to authenticated user
    match repo.get_character(&character_id, &claims.sub).await {
        Ok(_) => {}
        Err(_) => return Err(AppError::character_not_found(&character_id)),
    }

    match repo.get_character_unlocked_abilities(&character_id).await {
        Ok(abilities) => {
            let owned_abilities = abilities
                .into_iter()
                .map(|(ability, level)| (ability.clone(), level))
                .collect();
            Ok(Json(owned_abilities))
        }
        Err(e) => {
            tracing::error!("Failed to get character unlocked abilities: {:?}", e);
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
        Ok(()) => Ok(Json(serde_json::json!({
            "message": "Character last played updated",
            "timestamp": chrono::Utc::now().to_rfc3339()
        }))),
        Err(e) => {
            tracing::error!("Failed to update character last played: {:?}", e);
            Err(AppError::from(e))
        }
    }
}

/// Use a noncombat ability (activate buff)
pub async fn use_noncombat_ability(
    State(repo): State<Arc<UserRepository>>,
    Path((character_id, ability_id)): Path<(String, String)>,
    AuthClaims(claims): AuthClaims,
) -> Result<Json<Character>, AppError> {
    // Verify character belongs to authenticated user
    let mut character = repo
        .get_character(&character_id, &claims.sub)
        .await
        .map_err(|_| AppError::character_not_found(&character_id))?;

    // Get the ability
    let ability = get_ability_by_id(&ability_id)
        .map_err(|_| AppError::BadRequest("Ability not found".to_string()))?;

    // Verify it's a noncombat ability
    if ability.ability_type != AbilityType::NonCombat {
        return Err(AppError::BadRequest(
            "Only noncombat abilities can be used this way".to_string(),
        ));
    }

    // Check if character has enough mana
    let mana_cost = ability.mana_cost.unwrap_or(0);
    if character.mana < mana_cost {
        return Err(AppError::BadRequest(format!(
            "Not enough mana. Required: {}, Available: {}",
            mana_cost, character.mana
        )));
    }

    // Deduct mana
    character.mana -= mana_cost;

    // Check if ability is already active - if so, extend duration instead of adding new buff
    let duration = ability.duration.unwrap_or(1);
    let mut buff_extended = false;

    if let Some(active_buffs) = &mut character.active_buffs {
        if let Some(existing_buff) = active_buffs
            .iter_mut()
            .find(|buff| buff.ability_id == ability_id)
        {
            existing_buff.remaining_adventures += duration;
            buff_extended = true;
        }
    }

    // If buff wasn't extended, add it as new
    if !buff_extended {
        let new_buff = ActiveBuff {
            ability_id: ability_id.clone(),
            ability_name: ability.name.to_string(),
            remaining_adventures: duration,
        };

        if let Some(active_buffs) = &mut character.active_buffs {
            active_buffs.push(new_buff);
        } else {
            character.active_buffs = Some(vec![new_buff]);
        }
    }

    // Save character
    repo.update_character_state(&character_id, &character)
        .await
        .map_err(|e| {
            tracing::error!("Failed to update character after using ability: {:?}", e);
            AppError::from(e)
        })?;

    // Recalculate stats with the new buff before returning
    repo.populate_calculated_stats(&mut character)
        .await
        .map_err(AppError::from)?;

    Ok(Json(character))
}

#[derive(Debug, Deserialize)]
pub struct GrantExperienceRequest {
    pub amount: i64,
}

#[derive(Debug, Serialize)]
pub struct GrantExperienceResponse {
    pub character: Character,
    #[serde(rename = "levelsGained")]
    pub levels_gained: Vec<i64>,
    #[serde(rename = "experienceGranted")]
    pub experience_granted: i64,
}

/// Grant experience to a character
/// This will automatically handle level-ups and stat increases
pub async fn grant_experience(
    State(repo): State<Arc<UserRepository>>,
    Path(character_id): Path<String>,
    AuthClaims(claims): AuthClaims,
    Json(payload): Json<GrantExperienceRequest>,
) -> Result<Json<GrantExperienceResponse>, AppError> {
    // Verify character belongs to authenticated user
    let characters = repo
        .get_user_characters(&claims.sub)
        .await
        .map_err(|e| AppError::from(e))?;

    let _character = characters
        .iter()
        .find(|c| c.id == character_id)
        .ok_or_else(|| AppError::character_not_found(&character_id))?;

    // Validate experience amount
    if payload.amount <= 0 {
        return Err(AppError::validation_error(
            "Experience amount must be positive",
        ));
    }

    // Grant experience and get updated character + levels gained
    match repo.grant_experience(&character_id, payload.amount).await {
        Ok((updated_character, levels_gained)) => {
            let response = GrantExperienceResponse {
                character: updated_character,
                levels_gained: levels_gained.clone(),
                experience_granted: payload.amount,
            };

            Ok(Json(response))
        }
        Err(e) => {
            tracing::error!("Failed to grant experience: {:?}", e);
            Err(AppError::from(e))
        }
    }
}
