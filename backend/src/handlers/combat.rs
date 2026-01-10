use crate::damage_calculator::DamageCalculator;
use crate::error::AppError;
use crate::level_system::{
    calculate_level_from_experience, calculate_stat_increases_for_level, experience_for_level,
};
use crate::middleware::AuthClaims;
use crate::models::{Character, CharacterAbility};
use crate::repository::UserRepository;
use axum::{
    extract::{Path, State},
    Json,
};
use serde::{Deserialize, Serialize};
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

// Rest - restore character HP and MP at the cost of 1 adventure
pub async fn rest_character(
    State(repo): State<Arc<UserRepository>>,
    Path(character_id): Path<String>,
    AuthClaims(claims): AuthClaims,
) -> Result<Json<Character>, AppError> {
    // Get all user characters to verify ownership
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

    // Check if already at full HP and MP
    if character.stats.health >= character.stats.max_health
        && character.stats.mana >= character.stats.max_mana
    {
        return Err(AppError::validation_error(
            "Already at full health and mana",
        ));
    }

    // Decrease adventures by 1
    let new_adventures = character.stats.adventures - 1;

    // Update character adventures
    repo.update_character_adventures_with_user(&character_id, &claims.sub, new_adventures)
        .await
        .map_err(|e| {
            tracing::error!("Failed to update adventures: {:?}", e);
            AppError::from(e)
        })?;

    // Restore HP and MP to maximum
    repo.update_character_health(&character_id, &claims.sub, character.stats.max_health)
        .await
        .map_err(|e| {
            tracing::error!("Failed to update health: {:?}", e);
            AppError::from(e)
        })?;

    // Update mana (need to add this function to repository)
    repo.update_character_mana(&character_id, &claims.sub, character.stats.max_mana)
        .await
        .map_err(|e| {
            tracing::error!("Failed to update mana: {:?}", e);
            AppError::from(e)
        })?;

    // Get updated character
    let updated_character = repo
        .get_character(&character_id, &claims.sub)
        .await
        .map_err(|e| AppError::from(e))?;

    tracing::info!("Character {} rested and restored HP/MP", character_id);
    Ok(Json(updated_character))
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AttackResult {
    pub attacks: Vec<SingleAttack>,
    pub total_damage: i32,
    pub enemy_health: i32,
    pub enemy_attacks: Vec<SingleAttack>,
    pub player_health: i32,
    pub victory: bool,
    pub experience_gained: Option<i32>,
    pub victory_message: Option<String>,
    pub level_up: Option<LevelUpInfo>,
    pub game_state: Option<serde_json::Value>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LevelUpInfo {
    pub new_level: i64,
    pub stat_increases: StatIncreases,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StatIncreases {
    pub might: i64,
    pub defense: i64,
    pub magic: i64,
    pub resistance: i64,
    pub agility: i64,
    pub max_health: i64,
    pub max_mana: i64,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SingleAttack {
    pub damage: i32,
    pub description: String,
    pub is_dual_wield: bool,
}

// Perform a melee attack
pub async fn perform_attack(
    State(repo): State<Arc<UserRepository>>,
    Path(character_id): Path<String>,
    AuthClaims(claims): AuthClaims,
) -> Result<Json<AttackResult>, AppError> {
    // Get character directly to ensure we have the latest game_state
    let character = repo
        .get_character(&character_id, &claims.sub)
        .await
        .map_err(|_| AppError::character_not_found(&character_id))?;

    // Get character abilities to check for dual_wield passive
    let character_abilities: Vec<CharacterAbility> = repo
        .get_character_abilities(&character_id)
        .await
        .map_err(|e| AppError::from(e))?;

    // Fetch full ability data for each character ability
    let mut abilities = Vec::new();
    for char_ability in character_abilities {
        if let Ok(ability) = repo.get_ability(&char_ability.ability_id).await {
            abilities.push(ability);
        }
    }

    let has_dual_wield = abilities.iter().any(|ability| {
        ability.ability_type == "passive" && ability.passive_effect.as_deref() == Some("dual_wield")
    });

    let mut attacks = Vec::new();

    // Calculate base damage from might with ±10% variance
    let base_damage = character.stats.might;

    // First attack (main hand) - use DamageCalculator
    let damage = DamageCalculator::calculate_melee_attack(base_damage);

    attacks.push(SingleAttack {
        damage,
        description: format!(
            "You swing your weapon at the enemy, dealing **{}** damage!",
            damage
        ),
        is_dual_wield: false,
    });

    // Second attack if dual wielding
    if has_dual_wield {
        let damage = DamageCalculator::calculate_melee_attack(base_damage);

        attacks.push(SingleAttack {
            damage,
            description: format!(
                "Your off-hand weapon strikes true, dealing **{}** damage!",
                damage
            ),
            is_dual_wield: true,
        });
    }

    let total_damage: i32 = attacks.iter().map(|a| a.damage).sum();

    // Get current game state to update enemy health
    let game_state_str = character
        .game_state
        .as_ref()
        .ok_or_else(|| AppError::validation_error("No active combat"))?;

    let mut game_state: serde_json::Value = serde_json::from_str(game_state_str)
        .map_err(|e| AppError::validation_error(&format!("Invalid game state: {}", e)))?;

    // Increment turn number
    let current_turn = game_state
        .get("turnNumber")
        .and_then(|v| v.as_i64())
        .unwrap_or(1);
    let new_turn = current_turn + 1;
    game_state["turnNumber"] = serde_json::json!(new_turn);

    // Update enemy health
    let enemy = game_state
        .get_mut("enemy")
        .ok_or_else(|| AppError::validation_error("No enemy in game state"))?;

    let current_health = enemy
        .get("health")
        .and_then(|v| v.as_i64())
        .ok_or_else(|| AppError::validation_error("Invalid enemy health"))?
        as i32;

    let new_health = (current_health - total_damage).max(0);
    enemy["health"] = serde_json::json!(new_health);

    // Check for victory
    let victory = new_health <= 0;
    let mut experience_gained = None;
    let mut victory_message = None;
    let mut level_up = None;

    // Get enemy details for experience calculation (before cloning game_state)
    let enemy_level = enemy.get("level").and_then(|v| v.as_i64()).unwrap_or(1) as i32;
    let enemy_name = enemy
        .get("name")
        .and_then(|v| v.as_str())
        .unwrap_or("enemy")
        .to_string();

    // Get enemy might for counterattack (before accessing game_state again)
    let enemy_might = enemy
        .get("stats")
        .and_then(|s| s.get("might"))
        .and_then(|v| v.as_i64())
        .unwrap_or(10);

    // Creature counterattack if it survived
    let mut enemy_attacks = Vec::new();
    let mut player_health = game_state
        .get("playerHealth")
        .and_then(|v| v.as_i64())
        .unwrap_or(character.stats.health) as i32;

    if !victory {
        // Calculate counterattack damage with ±10% variance
        let counter_damage = DamageCalculator::calculate_melee_attack(enemy_might);

        enemy_attacks.push(SingleAttack {
            damage: counter_damage,
            description: format!(
                "{} counterattacks, dealing **{}** damage!",
                enemy_name, counter_damage
            ),
            is_dual_wield: false,
        });

        // Update player health
        player_health = (player_health - counter_damage).max(0);
        game_state["playerHealth"] = serde_json::json!(player_health);

        // Update character health in database
        repo.update_character_health(&character_id, &claims.sub, player_health as i64)
            .await
            .map_err(|e| {
                tracing::error!("Failed to update character health: {:?}", e);
                AppError::from(e)
            })?;
    }

    // Store the updated game state (with enemy at 0 health) to return to frontend
    let updated_game_state = game_state.clone();

    if victory {
        // Calculate experience reward (base 50 + 25 per enemy level)
        let exp_reward = 50 + (enemy_level * 25);
        experience_gained = Some(exp_reward);

        // Award experience to character
        // First reconstruct total accumulated experience from current level + experience into level
        let mut total_accumulated_exp = 0;
        for lvl in 2..=character.level {
            total_accumulated_exp += experience_for_level(lvl);
        }
        total_accumulated_exp += character.experience; // Add current progress into level

        let new_total_experience = total_accumulated_exp + exp_reward as i64;

        // Check for level up
        let old_level = character.level;
        let (new_level, exp_into_level, exp_for_next) =
            calculate_level_from_experience(new_total_experience);

        // Update experience (store experience into level) and experience_to_next (for XP bar)
        repo.update_character_experience_progress(
            &character_id,
            &claims.sub,
            exp_into_level, // Store XP into current level, not total
            exp_for_next,
        )
        .await
        .map_err(|e| AppError::from(e))?;

        // Update level if it changed
        if new_level > old_level {
            repo.update_character_level(&character_id, &claims.sub, new_level)
                .await
                .map_err(|e| AppError::from(e))?;

            // Calculate stat increases
            let (might, defense, magic, resistance, agility, max_health, max_mana) =
                calculate_stat_increases_for_level(&character.class_id, new_level);

            // Apply stat increases to character
            repo.apply_stat_increases(
                &character_id,
                &claims.sub,
                might,
                defense,
                magic,
                resistance,
                agility,
                max_health,
                max_mana,
            )
            .await
            .map_err(|e| AppError::from(e))?;

            level_up = Some(LevelUpInfo {
                new_level,
                stat_increases: StatIncreases {
                    might,
                    defense,
                    magic,
                    resistance,
                    agility,
                    max_health,
                    max_mana,
                },
            });
        }

        // Subtract one adventure
        let new_adventures = (character.stats.adventures - 1).max(0);
        repo.update_character_adventures_with_user(&character_id, &claims.sub, new_adventures)
            .await
            .map_err(|e| AppError::from(e))?;

        // Clear game state (combat is over)
        repo.update_character_game_state(&character_id, &claims.sub, None)
            .await
            .map_err(|e| AppError::from(e))?;

        victory_message = Some(format!(
            "Victory! You have defeated {}! You gained **{}** experience.",
            enemy_name, exp_reward
        ));

        tracing::info!(
            "Character {} defeated {} and gained {} experience",
            character_id,
            enemy_name,
            exp_reward
        );
    } else {
        // Save updated game state
        let updated_game_state_str = serde_json::to_string(&game_state).map_err(|e| {
            AppError::validation_error(&format!("Failed to serialize game state: {}", e))
        })?;

        repo.update_character_game_state(&character_id, &claims.sub, Some(updated_game_state_str))
            .await
            .map_err(|e| AppError::from(e))?;

        tracing::info!(
            "Character {} attacked for {} total damage (dual_wield: {}), enemy health: {} -> {}",
            character_id,
            total_damage,
            has_dual_wield,
            current_health,
            new_health
        );
    }

    Ok(Json(AttackResult {
        attacks,
        total_damage,
        enemy_health: new_health,
        enemy_attacks,
        player_health,
        victory,
        experience_gained,
        victory_message,
        level_up,
        game_state: Some(updated_game_state),
    }))
}
