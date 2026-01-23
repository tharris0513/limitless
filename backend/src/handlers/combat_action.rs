use crate::error::AppError;
use crate::middleware::AuthClaims;
use crate::models::{CharacterAbility, CreatureInCombat};
use crate::repository::UserRepository;
use axum::{
    extract::{Path, State},
    Json,
};
use std::sync::Arc;

use crate::combat::{
    apply_enemy_attack, decrement_buff_durations, handle_defeat, handle_victory,
    process_ability_attacks, process_melee_attacks, update_cooldowns, AbilityProcessor,
    CombatActionRequest, CombatActionResult, GameStateHelper, GS_ABILITY_COOLDOWNS, GS_ENEMY,
    GS_PLAYER_HEALTH, GS_STATUS, GS_TURN_NUMBER,
};

use crate::combat::helpers::GS_ENEMY_HEALTH;

// Save character's game state
pub async fn save_game_state(
    State(repo): State<Arc<UserRepository>>,
    Path(character_id): Path<String>,
    AuthClaims(claims): AuthClaims,
    Json(game_state): Json<serde_json::Value>,
) -> Result<Json<serde_json::Value>, AppError> {
    // Verify character ownership
    repo.get_character(&character_id, &claims.sub)
        .await
        .map_err(|_| AppError::character_not_found("Character not found or access denied"))?;

    // Convert game state to JSON string
    let game_state_str = serde_json::to_string(&game_state)
        .map_err(|e| AppError::validation_error(&format!("Invalid game state JSON: {}", e)))?;

    match repo
        .update_character_game_state(&character_id, &claims.sub, Some(game_state_str))
        .await
    {
        Ok(()) => Ok(Json(serde_json::json!({
            "message": "Game state saved",
            "timestamp": chrono::Utc::now().to_rfc3339()
        }))),
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
    // Verify character ownership
    repo.get_character(&character_id, &claims.sub)
        .await
        .map_err(|_| AppError::character_not_found("Character not found or access denied"))?;

    match repo
        .update_character_game_state(&character_id, &claims.sub, None)
        .await
    {
        Ok(()) => Ok(Json(serde_json::json!({
            "message": "Game state cleared",
            "timestamp": chrono::Utc::now().to_rfc3339()
        }))),
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
) -> Result<Json<crate::models::Character>, AppError> {
    // Get all user characters to verify ownership and get current adventures
    let characters = repo
        .get_user_characters(&claims.sub)
        .await
        .map_err(AppError::from)?;

    let mut character = characters
        .iter()
        .find(|c| c.id == character_id)
        .cloned()
        .ok_or_else(|| AppError::character_not_found(&character_id))?;

    // Check if character has adventures available
    if character.adventures <= 0 {
        return Err(AppError::validation_error("No adventures remaining"));
    }

    // Decrease adventures by 1
    let new_adventures = character.adventures - 1;

    // Decrement buff durations and remove expired buffs
    character.active_buffs = decrement_buff_durations(character.active_buffs);

    // Update character adventures
    let _updated_character = repo
        .update_character_adventures(&character_id, new_adventures)
        .await
        .map_err(|e| {
            tracing::error!("Failed to update adventures: {:?}", e);
            AppError::from(e)
        })?;

    // Update active buffs
    repo.update_character_state(&character_id, &character)
        .await
        .map_err(|e| {
            tracing::error!("Failed to update active buffs: {:?}", e);
            AppError::from(e)
        })?;

    // Clear game state
    repo.update_character_game_state(&character_id, &claims.sub, None)
        .await
        .map_err(|e| {
            tracing::error!("Failed to clear game state: {:?}", e);
            AppError::from(e)
        })?;

    // Fetch the character again to get the cleared game state
    let final_character = repo
        .get_user_characters(&claims.sub)
        .await
        .map_err(AppError::from)?
        .into_iter()
        .find(|c| c.id == character_id)
        .ok_or_else(|| AppError::character_not_found(&character_id))?;
    Ok(Json(final_character))
}

// Rest - restore character HP and MP at the cost of 1 adventure
pub async fn rest_character(
    State(repo): State<Arc<UserRepository>>,
    Path(character_id): Path<String>,
    AuthClaims(claims): AuthClaims,
) -> Result<Json<crate::models::Character>, AppError> {
    // Get all user characters to verify ownership
    let characters = repo
        .get_user_characters(&claims.sub)
        .await
        .map_err(AppError::from)?;

    let mut character = characters
        .iter()
        .find(|c| c.id == character_id)
        .cloned()
        .ok_or_else(|| AppError::character_not_found(&character_id))?;

    // Check if character has adventures available
    if character.adventures <= 0 {
        return Err(AppError::validation_error("No adventures remaining"));
    }

    // Check if already at full HP and MP
    if character.health >= character.max_health && character.mana >= character.max_mana {
        return Err(AppError::validation_error(
            "Already at full health and mana",
        ));
    }

    // Decrease adventures by 1
    let new_adventures = character.adventures - 1;

    // Decrement buff durations and remove expired buffs
    character.active_buffs = decrement_buff_durations(character.active_buffs);

    // Update character adventures
    repo.update_character_adventures_with_user(&character_id, &claims.sub, new_adventures)
        .await
        .map_err(|e| {
            tracing::error!("Failed to update adventures: {:?}", e);
            AppError::from(e)
        })?;

    // Restore HP and MP to maximum
    repo.update_character_health(&character_id, &claims.sub, character.max_health)
        .await
        .map_err(|e| {
            tracing::error!("Failed to update health: {:?}", e);
            AppError::from(e)
        })?;

    // Update mana
    repo.update_character_mana(&character_id, &claims.sub, character.max_mana)
        .await
        .map_err(|e| {
            tracing::error!("Failed to update mana: {:?}", e);
            AppError::from(e)
        })?;

    // Update active buffs
    repo.update_character_state(&character_id, &character)
        .await
        .map_err(|e| {
            tracing::error!("Failed to update active buffs: {:?}", e);
            AppError::from(e)
        })?;

    // Get updated character
    let updated_character = repo
        .get_character(&character_id, &claims.sub)
        .await
        .map_err(AppError::from)?;
    Ok(Json(updated_character))
}

/// Unified combat action handler - handles both melee attacks and ability usage
pub async fn perform_combat_action(
    State(repo): State<Arc<UserRepository>>,
    Path(character_id): Path<String>,
    AuthClaims(claims): AuthClaims,
    Json(action): Json<CombatActionRequest>,
) -> Result<Json<CombatActionResult>, AppError> {
    // Get character
    let mut character = repo
        .get_character(&character_id, &claims.sub)
        .await
        .map_err(|_| AppError::character_not_found(&character_id))?;

    // Validate active combat
    let game_state_str = character
        .game_state
        .as_ref()
        .ok_or_else(|| AppError::validation_error("No active combat"))?;

    let mut game_state: serde_json::Value = serde_json::from_str(game_state_str)
        .map_err(|e| AppError::validation_error(&format!("Invalid game state: {}", e)))?;

    // Get character abilities for passive checks and ability validation
    let character_abilities: Vec<CharacterAbility> = repo
        .get_character_abilities(&character_id)
        .await
        .map_err(AppError::from)?;

    let mut abilities = Vec::new();
    for char_ability in character_abilities {
        if let Ok(ability) = repo.get_ability(&char_ability.ability_id).await {
            abilities.push(ability);
        }
    }

    // Create ability processor
    let processor = AbilityProcessor::new(&character, &abilities);

    let enemy: CreatureInCombat = serde_json::from_value(
        game_state
            .get(GS_ENEMY)
            .ok_or_else(|| AppError::validation_error("No enemy in game state"))?
            .clone(),
    )
    .map_err(|e| {
        AppError::validation_error(&format!("Failed to parse enemy from game state: {}", e))
    })?;

    // === Generate player attacks based on action type ===
    let (attacks, mana_cost) = match action {
        CombatActionRequest::Melee => {
            let attacks = process_melee_attacks(&processor, &character, &game_state, enemy.defense);
            (attacks, 0i64)
        }
        CombatActionRequest::Ability { ref ability_id } => {
            // Validate character owns this ability
            let ability = processor
                .find_ability(ability_id)
                .ok_or_else(|| AppError::validation_error("Ability not found or not unlocked"))?;

            // Check if ability is on cooldown
            if let Some(cooldown) = ability.cooldown {
                if cooldown > 0 {
                    if let Some(cooldowns) = game_state.get(GS_ABILITY_COOLDOWNS) {
                        if let Some(cooldown_turns) =
                            cooldowns.get(&ability.id).and_then(|v| v.as_i64())
                        {
                            if cooldown_turns > 0 {
                                return Err(AppError::validation_error(&format!(
                                    "Ability is on cooldown for {} more turn{}",
                                    cooldown_turns,
                                    if cooldown_turns == 1 { "" } else { "s" }
                                )));
                            }
                        }
                    }
                }
            }

            // Check mana cost
            let mana_cost_required = ability.mana_cost.unwrap_or(0);
            if character.mana < mana_cost_required {
                return Err(AppError::validation_error(&format!(
                    "Not enough mana. Required: {}, Available: {}",
                    mana_cost_required, character.mana
                )));
            }

            // Process ability effects
            let attacks =
                process_ability_attacks(ability, &character, &processor, &mut game_state)?;

            (attacks, mana_cost_required)
        }
    };

    let total_damage: i64 = attacks.iter().map(|a| a.damage).sum();

    // === Apply combat round changes ===

    // Apply mana cost
    character.mana -= mana_cost;

    // Increment turn number
    let gs_helper = GameStateHelper::new(&game_state);
    let current_turn = gs_helper.turn_number();
    let new_turn = current_turn + 1;
    game_state[GS_TURN_NUMBER] = serde_json::json!(new_turn);

    // Update combat status based on turn number (if not already finished)
    if new_turn == 1 {
        game_state[GS_STATUS] = serde_json::json!("started");
    } else if new_turn > 1 {
        game_state[GS_STATUS] = serde_json::json!("ongoing");
    }

    // Update ability cooldowns
    let cooldowns = update_cooldowns(&mut game_state, &action, &abilities);

    // Update enemy health
    let enemy: CreatureInCombat = serde_json::from_value(
        game_state
            .get(GS_ENEMY)
            .ok_or_else(|| AppError::validation_error("No enemy in game state"))?
            .clone(),
    )
    .map_err(|e| {
        AppError::validation_error(&format!("Failed to parse enemy from game state: {}", e))
    })?;

    let new_enemy_health = (enemy.health - total_damage).max(0);

    // Write updated enemy health into game_state for persistence and frontend
    if let Some(enemy_obj) = game_state.get_mut(GS_ENEMY) {
        if let Some(obj) = enemy_obj.as_object_mut() {
            obj.insert(
                GS_ENEMY_HEALTH.to_string(),
                serde_json::json!(new_enemy_health),
            );
        }
    }

    // === Handle combat outcome ===

    let victory = new_enemy_health <= 0;
    let gs_helper = GameStateHelper::new(&game_state);
    let mut player_health = gs_helper.player_health(character.health);

    let (experience_gained, victory_message, level_up, enemy_attacks, defeat, defeat_message) =
        if victory {
            let (exp, vic_msg, lvl_up) = handle_victory(
                &repo,
                &character_id,
                &claims.sub,
                &mut character,
                enemy.name,
                enemy.experience_reward,
            )
            .await?;

            (exp, vic_msg, lvl_up, Vec::new(), false, None)
        } else {
            // Enemy counterattacks (with player defense mitigation)
            let (attacks, counter_damage) = apply_enemy_attack(
                enemy.might,
                character.defense,
                &enemy.name,
                enemy.attack_description,
            );

            // Update player health
            player_health = (player_health - counter_damage).max(0);
            game_state[GS_PLAYER_HEALTH] = serde_json::json!(player_health);
            character.health = player_health as i64;

            // Update character health in database
            repo.update_character_health(&character_id, &claims.sub, character.health)
                .await
                .map_err(AppError::from)?;

            // Check for defeat
            let (defeat, defeat_msg) = if player_health <= 0 {
                let msg = handle_defeat(&repo, &character_id, &claims.sub, &mut character, &enemy.name)
                    .await?;
                (true, msg)
            } else {
                (false, None)
            };

            (None, None, None, attacks, defeat, defeat_msg)
        };

    // Update mana if it was consumed (ability use)
    if mana_cost > 0 && !victory && !defeat {
        repo.update_character_mana(&character_id, &claims.sub, character.mana)
            .await
            .map_err(AppError::from)?;
    }

    // Prepare game state for response
    let mut response_game_state = game_state.clone();

    // Update combat status when it ends
    if victory {
        response_game_state[GS_STATUS] = serde_json::json!("victory");
    } else if defeat {
        response_game_state[GS_STATUS] = serde_json::json!("defeat");
    }

    // Save updated game state if combat continues
    if !victory && !defeat {
        let updated_game_state_str = serde_json::to_string(&game_state).map_err(|e| {
            AppError::validation_error(&format!("Failed to serialize game state: {}", e))
        })?;

        repo.update_character_game_state(&character_id, &claims.sub, Some(updated_game_state_str))
            .await
            .map_err(AppError::from)?;
    }

    Ok(Json(CombatActionResult {
        attacks,
        total_damage,
        enemy_health: new_enemy_health,
        enemy_attacks,
        player_health,
        player_mana: character.mana,
        victory,
        defeat,
        experience_gained,
        victory_message,
        defeat_message,
        level_up,
        game_state: Some(response_game_state),
        ability_cooldowns: cooldowns,
    }))
}
