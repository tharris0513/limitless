use crate::game::abilities::get_ability_by_id;
use crate::middleware::AuthClaims;
use crate::repository::UserRepository;
use crate::{error::AppError, game::abilities::CharacterAbility};
use axum::{
    extract::{Path, State},
    Json,
};
use std::sync::Arc;

use crate::combat::{
    apply_enemy_attack, decrement_buff_durations, handle_defeat, handle_victory,
    process_ability_attacks, process_melee_attacks, update_cooldowns, AbilityProcessor,
    CombatActionRequest, CombatActionResult, CombatOutcome, CombatState,
};

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
) -> Result<Json<crate::models::Character>, AppError> {
    // Verify character ownership
    repo.get_character(&character_id, &claims.sub)
        .await
        .map_err(|_| AppError::character_not_found("Character not found or access denied"))?;

    // Clear game state (buffs were already decremented in handle_victory/handle_defeat)
    repo.update_character_game_state(&character_id, &claims.sub, None)
        .await
        .map_err(|e| {
            tracing::error!("Failed to clear game state: {:?}", e);
            AppError::from(e)
        })?;

    // Fetch the character again to get the cleared game state and updated calculated stats
    let final_character = repo
        .get_character(&character_id, &claims.sub)
        .await
        .map_err(AppError::from)?;

    Ok(Json(final_character))
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

    // Get calculated max values (or base values if not calculated)
    let max_health = character
        .calculated_stats
        .as_ref()
        .map(|s| s.max_health)
        .unwrap_or(character.max_health);
    let max_mana = character
        .calculated_stats
        .as_ref()
        .map(|s| s.max_mana)
        .unwrap_or(character.max_mana);

    // Check if already at full HP and MP
    if character.health >= max_health && character.mana >= max_mana {
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

    // Update character's health and mana in memory before saving
    character.health = max_health;
    character.mana = max_mana;

    // Restore HP and MP to calculated maximum (includes buff bonuses)
    repo.update_character_health(&character_id, &claims.sub, max_health)
        .await
        .map_err(|e| {
            tracing::error!("Failed to update health: {:?}", e);
            AppError::from(e)
        })?;

    // Update mana to calculated maximum (includes buff bonuses)
    repo.update_character_mana(&character_id, &claims.sub, max_mana)
        .await
        .map_err(|e| {
            tracing::error!("Failed to update mana: {:?}", e);
            AppError::from(e)
        })?;

    // Update active buffs (this also updates mana, so we set it in memory first)
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

    let game_state_json: serde_json::Value = serde_json::from_str(game_state_str)
        .map_err(|e| AppError::validation_error(&format!("Invalid game state: {}", e)))?;

    tracing::debug!("Raw game_state JSON: {}", game_state_json);

    // Get player max health for combat state
    let player_max_health = character
        .calculated_stats
        .as_ref()
        .map(|s| s.max_health)
        .unwrap_or(character.max_health);

    // Parse into CombatState - this encapsulates all health-related operations
    let mut combat_state = CombatState::from_json(&game_state_json, player_max_health)?;

    tracing::debug!(
        "Combat state parsed - enemy: {} hp={}, player hp={}",
        combat_state.enemy_name(),
        combat_state.enemy_health(),
        combat_state.player_health()
    );

    // Get character abilities for passive checks and ability validation
    let character_abilities: Vec<CharacterAbility> = repo
        .get_character_abilities(&character_id)
        .await
        .map_err(AppError::from)?;

    let mut abilities = Vec::new();
    for char_ability in character_abilities {
        if let Ok(ability) = get_ability_by_id(&char_ability.ability_id) {
            abilities.push(ability);
        }
    }

    // Create ability processor
    let processor = AbilityProcessor::new(&character, &abilities);

    // We need a mutable JSON representation for ability processing (enchantments modify it)
    let mut game_state = combat_state.to_json();

    // === Generate player attacks based on action type ===
    let (attacks, mana_cost) = match action {
        CombatActionRequest::Melee => {
            let attacks = process_melee_attacks(
                &processor,
                &character,
                &game_state,
                combat_state.enemy_defense(),
            );
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
                    if let Some(&cooldown_turns) = combat_state.ability_cooldowns().get(ability.id)
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

            // Check mana cost
            let mana_cost_required = ability.mana_cost.unwrap_or(0);
            if character.mana < mana_cost_required {
                return Err(AppError::validation_error(&format!(
                    "Not enough mana. Required: {}, Available: {}",
                    mana_cost_required, character.mana
                )));
            }

            // Process ability effects (may modify game_state for enchantments)
            let attacks =
                process_ability_attacks(ability, &character, &processor, &mut game_state)?;

            (attacks, mana_cost_required)
        }
    };

    let total_damage: i64 = attacks.iter().map(|a| a.damage).sum();

    // === Apply combat round changes ===

    // Apply mana cost
    character.mana -= mana_cost;

    // Advance turn (automatically updates status)
    combat_state.advance_turn();

    // Update ability cooldowns
    let cooldowns = update_cooldowns(&mut game_state, &action, &abilities);

    // Sync cooldowns back to combat state
    *combat_state.ability_cooldowns_mut() = cooldowns.clone();

    // Sync any enchantment changes from ability processing back to combat state
    if let Some(primary) = game_state
        .get("primaryWeaponEnchanted")
        .and_then(|v| v.as_str())
    {
        combat_state.set_primary_weapon_enchanted(Some(primary.to_string()));
    }
    if let Some(secondary) = game_state
        .get("secondaryWeaponEnchanted")
        .and_then(|v| v.as_str())
    {
        combat_state.set_secondary_weapon_enchanted(Some(secondary.to_string()));
    }

    // === Apply damage to enemy and check for victory ===
    // This is the key observation point - damage_enemy automatically checks if health <= 0
    tracing::debug!(
        "Applying {} damage to enemy (current hp={})",
        total_damage,
        combat_state.enemy_health()
    );
    let enemy_outcome = combat_state.damage_enemy(total_damage);
    let victory = enemy_outcome == CombatOutcome::Victory;
    tracing::debug!(
        "After damage: enemy hp={}, outcome={:?}, victory={}",
        combat_state.enemy_health(),
        enemy_outcome,
        victory
    );

    let (experience_gained, victory_message, level_up, enemy_attacks, defeat, defeat_message) =
        if victory {
            // Update status to victory
            combat_state.update_status(CombatOutcome::Victory);

            let (exp, vic_msg, lvl_up) = handle_victory(
                &repo,
                &character_id,
                &claims.sub,
                &mut character,
                combat_state.enemy_name().to_string(),
                combat_state.enemy_experience_reward(),
            )
            .await?;

            (exp, vic_msg, lvl_up, Vec::new(), false, None)
        } else {
            // Enemy counterattacks (with player defense mitigation)
            let defense = character
                .calculated_stats
                .as_ref()
                .map(|s| s.defense)
                .unwrap_or(character.defense);
            let (attacks, counter_damage) = apply_enemy_attack(
                combat_state.enemy_might(),
                defense,
                combat_state.enemy_name(),
                combat_state.enemy_attack_description().to_string(),
            );

            // Apply damage to player and check for defeat
            // This is the key observation point - damage_player automatically checks if health <= 0
            let player_outcome = combat_state.damage_player(counter_damage);
            let defeat = player_outcome == CombatOutcome::Defeat;

            // Sync player health to character
            character.health = combat_state.player_health();

            // Update character health in database
            repo.update_character_health(&character_id, &claims.sub, character.health)
                .await
                .map_err(AppError::from)?;

            // Handle defeat if player health reached 0
            let defeat_msg = if defeat {
                combat_state.update_status(CombatOutcome::Defeat);
                let enemy_name = combat_state.enemy_name().to_string();
                handle_defeat(
                    &repo,
                    &character_id,
                    &claims.sub,
                    &mut character,
                    &enemy_name,
                )
                .await?
            } else {
                None
            };

            (None, None, None, attacks, defeat, defeat_msg)
        };

    // Update mana if it was consumed (ability use)
    if mana_cost > 0 && !victory && !defeat {
        repo.update_character_mana(&character_id, &claims.sub, character.mana)
            .await
            .map_err(AppError::from)?;
    }

    // Prepare game state for response (serialize from CombatState)
    let response_game_state = combat_state.to_json();

    tracing::debug!(
        "Response: victory={}, defeat={}, enemy_hp={}, player_hp={}, status={}",
        victory,
        defeat,
        combat_state.enemy_health(),
        combat_state.player_health(),
        response_game_state
            .get("status")
            .and_then(|v| v.as_str())
            .unwrap_or("unknown")
    );

    // Save updated game state if combat continues
    if !victory && !defeat {
        let updated_game_state_str = serde_json::to_string(&response_game_state).map_err(|e| {
            AppError::validation_error(&format!("Failed to serialize game state: {}", e))
        })?;

        repo.update_character_game_state(&character_id, &claims.sub, Some(updated_game_state_str))
            .await
            .map_err(AppError::from)?;
    }

    Ok(Json(CombatActionResult {
        attacks,
        total_damage,
        enemy_health: combat_state.enemy_health(),
        enemy_attacks,
        player_health: combat_state.player_health(),
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
