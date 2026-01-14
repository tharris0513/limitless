use crate::damage_calculator::DamageCalculator;
use crate::error::AppError;
use crate::level_system::{
    calculate_level_from_experience, calculate_stat_increases_for_level, experience_for_level,
};
use crate::middleware::AuthClaims;
use crate::models::CharacterAbility;
use crate::repository::UserRepository;
use axum::{
    extract::{Path, State},
    Json,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UseAbilityRequest {
    pub ability_id: String,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct UseAbilityResult {
    pub attacks: Vec<SingleAttack>,
    pub total_damage: i32,
    pub enemy_health: i32,
    pub player_health: i32,
    pub player_mana: i32,
    pub victory: bool,
    pub defeat: bool,
    pub experience_gained: Option<i32>,
    pub victory_message: Option<String>,
    pub defeat_message: Option<String>,
    pub level_up: Option<LevelUpInfo>,
    pub game_state: Option<serde_json::Value>,
    pub enemy_attacks: Vec<SingleAttack>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SingleAttack {
    pub damage: i32,
    pub description: String,
    pub is_dual_wield: bool,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LevelUpInfo {
    pub new_level: i64,
    pub stat_increases: StatIncreases,
}

#[derive(Debug, Serialize)]
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

/// Parse attack description template, replacing placeholders with actual values
fn parse_attack_description(template: &str, damage: i32, ability_name: &str) -> String {
    template
        .replace("${damage}", &damage.to_string())
        .replace("${x}", &damage.to_string())
        .replace("${name}", ability_name)
}

pub async fn use_ability(
    State(repo): State<Arc<UserRepository>>,
    Path(character_id): Path<String>,
    AuthClaims(claims): AuthClaims,
    Json(request): Json<UseAbilityRequest>,
) -> Result<Json<UseAbilityResult>, AppError> {
    // Get character
    let mut character = repo
        .get_character(&character_id, &claims.sub)
        .await
        .map_err(|_| AppError::character_not_found(&character_id))?;

    // Get ability
    let ability = repo.get_ability(&request.ability_id).await.map_err(|_| {
        AppError::validation_error(&format!("Ability {} not found", request.ability_id))
    })?;

    // Verify character has this ability
    let character_abilities: Vec<CharacterAbility> = repo
        .get_character_abilities(&character_id)
        .await
        .map_err(|e| AppError::from(e))?;

    let has_ability = character_abilities
        .iter()
        .any(|ca| ca.ability_id == request.ability_id);

    if !has_ability {
        return Err(AppError::validation_error(
            "Character does not have this ability",
        ));
    }

    // Check if ability is active
    if ability.ability_type != "active" {
        return Err(AppError::validation_error(
            "Only active abilities can be used in combat",
        ));
    }

    // Check mana cost
    if character.stats.mana < ability.mana_cost {
        return Err(AppError::validation_error("Not enough mana"));
    }

    // TODO: Check cooldown when cooldown system is implemented

    // Get current game state
    let game_state_str = character
        .game_state
        .as_ref()
        .ok_or_else(|| AppError::validation_error("No active combat"))?;

    let mut game_state: serde_json::Value = serde_json::from_str(game_state_str)
        .map_err(|e| AppError::validation_error(&format!("Invalid game state: {}", e)))?;

    // Calculate damage using formula
    let damage = if ability.damage_formula.is_some() {
        DamageCalculator::calculate_damage(&ability, &character, None)
            .map_err(|e| AppError::validation_error(&format!("Damage calculation failed: {}", e)))?
    } else {
        0
    };

    // Create attack description
    let attack_description = if let Some(template) = &ability.attack_description {
        parse_attack_description(template, damage, &ability.name)
    } else {
        format!(
            "You use **{}**, dealing **{}** damage!",
            ability.name, damage
        )
    };

    let attacks = vec![SingleAttack {
        damage,
        description: attack_description,
        is_dual_wield: false,
    }];

    // Deduct mana
    character.stats.mana = (character.stats.mana - ability.mana_cost).max(0);

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

    let new_health = (current_health - damage).max(0);
    enemy["health"] = serde_json::json!(new_health);

    // Check for victory
    let victory = new_health <= 0;
    let mut experience_gained = None;
    let mut victory_message = None;
    let mut level_up = None;

    // Get enemy details for experience calculation
    let enemy_level = enemy.get("level").and_then(|v| v.as_i64()).unwrap_or(1) as i32;
    let enemy_name = enemy
        .get("name")
        .and_then(|v| v.as_str())
        .unwrap_or("enemy")
        .to_string();

    let enemy_exp_reward = enemy
        .get("experienceReward")
        .and_then(|v| v.as_i64())
        .unwrap_or(50) as i32;

    let mut enemy_attacks = Vec::new();
    let mut defeat = false;

    if victory {
        // Enemy defeated - award experience
        experience_gained = Some(enemy_exp_reward);
        victory_message = Some(format!(
            "You have slain {}! You gain **{}** experience.",
            enemy_name, enemy_exp_reward
        ));

        // Award experience and check for level up
        character.experience += enemy_exp_reward as i64;

        let old_level = character.level;
        let (new_level, _, _) = calculate_level_from_experience(character.experience);

        if new_level > old_level {
            // Level up!
            repo.update_character_level(&character_id, &claims.sub, new_level)
                .await
                .map_err(|e| AppError::from(e))?;

            // Calculate and apply stat increases
            let (might, defense, magic, resistance, agility, max_health, max_mana) =
                calculate_stat_increases_for_level(&character.class_id, new_level);

            // Apply stat increases
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

            // Unlock new abilities for this level
            if let Err(e) = repo
                .unlock_character_abilities(&character_id, &character.class_id, new_level)
                .await
            {
                tracing::error!(
                    "Failed to unlock abilities for level {}: {:?}",
                    new_level,
                    e
                );
            }

            // Update character stats after level up for health/mana calculation
            character.stats.might += might;
            character.stats.defense += defense;
            character.stats.magic += magic;
            character.stats.resistance += resistance;
            character.stats.agility += agility;
            character.stats.max_health += max_health;
            character.stats.max_mana += max_mana;
            character.level = new_level;

            // Fully restore health and mana on level up
            character.stats.health = character.stats.max_health;
            character.stats.mana = character.stats.max_mana;
        }

        // Update experience progress
        let (_, exp_into_level, exp_for_next) =
            calculate_level_from_experience(character.experience);
        repo.update_character_experience_progress(
            &character_id,
            &claims.sub,
            exp_into_level,
            exp_for_next,
        )
        .await
        .map_err(|e| AppError::from(e))?;

        // Update health and mana (current values, not max - unless leveled up above)
        repo.update_character_health(&character_id, &claims.sub, character.stats.health)
            .await
            .map_err(|e| AppError::from(e))?;

        repo.update_character_mana(&character_id, &claims.sub, character.stats.mana)
            .await
            .map_err(|e| AppError::from(e))?;

        // Clear game state on victory
        repo.update_character_game_state(&character_id, &claims.sub, None)
            .await
            .map_err(|e| AppError::from(e))?;

        // Mark game state as finished for frontend
        game_state["finished"] = serde_json::json!(true);

        return Ok(Json(UseAbilityResult {
            attacks,
            total_damage: damage,
            enemy_health: new_health,
            player_health: character.stats.health as i32,
            player_mana: character.stats.mana as i32,
            victory: true,
            defeat: false,
            experience_gained,
            victory_message,
            defeat_message: None,
            level_up,
            game_state: Some(game_state),
            enemy_attacks: vec![],
        }));
    }

    // Enemy counterattacks if still alive
    let enemy_might = enemy.get("might").and_then(|v| v.as_i64()).unwrap_or(10) as i32;

    let enemy_damage = DamageCalculator::calculate_melee_attack(enemy_might as i64);
    let enemy_attack_template = enemy
        .get("attackDescription")
        .and_then(|v| v.as_str())
        .map(|s| s.to_string());

    let counter_damage = (enemy_damage - (character.stats.defense as i32 / 2)).max(1);

    let attack_description = if let Some(template) = enemy_attack_template {
        crate::handlers::combat::parse_attack_description(&template, counter_damage, &enemy_name)
    } else {
        format!(
            "The {} attacks for **{}** damage!",
            enemy_name, counter_damage
        )
    };

    enemy_attacks.push(SingleAttack {
        damage: counter_damage,
        description: attack_description,
        is_dual_wield: false,
    });

    character.stats.health = (character.stats.health - counter_damage as i64).max(0);

    // Check for defeat
    if character.stats.health <= 0 {
        defeat = true;
        let defeat_msg = format!(
            "💀 You have been defeated by the {}! You lose 1 adventure and are returned to safety.",
            enemy_name
        );

        // Deduct adventure
        let new_adventures = (character.stats.adventures - 1).max(0);
        repo.update_character_adventures_with_user(&character_id, &claims.sub, new_adventures)
            .await
            .map_err(|e| AppError::from(e))?;

        // Update health and mana
        repo.update_character_health(&character_id, &claims.sub, 0)
            .await
            .map_err(|e| AppError::from(e))?;

        repo.update_character_mana(&character_id, &claims.sub, character.stats.mana)
            .await
            .map_err(|e| AppError::from(e))?;

        // Clear game state on defeat
        repo.update_character_game_state(&character_id, &claims.sub, None)
            .await
            .map_err(|e| AppError::from(e))?;

        // Mark game state as finished for frontend
        game_state["finished"] = serde_json::json!(true);

        return Ok(Json(UseAbilityResult {
            attacks,
            total_damage: damage,
            enemy_health: new_health,
            player_health: 0,
            player_mana: character.stats.mana as i32,
            victory: false,
            defeat: true,
            experience_gained: None,
            victory_message: None,
            defeat_message: Some(defeat_msg),
            level_up: None,
            game_state: Some(game_state),
            enemy_attacks,
        }));
    }

    // Update character health and mana
    repo.update_character_health(&character_id, &claims.sub, character.stats.health)
        .await
        .map_err(|e| AppError::from(e))?;

    repo.update_character_mana(&character_id, &claims.sub, character.stats.mana)
        .await
        .map_err(|e| AppError::from(e))?;

    // Update game state with new enemy health and turn number
    let game_state_str = serde_json::to_string(&game_state).map_err(|e| {
        AppError::validation_error(&format!("Failed to serialize game state: {}", e))
    })?;

    repo.update_character_game_state(&character_id, &claims.sub, Some(game_state_str))
        .await
        .map_err(|e| AppError::from(e))?;

    Ok(Json(UseAbilityResult {
        attacks,
        total_damage: damage,
        enemy_health: new_health,
        player_health: character.stats.health as i32,
        player_mana: character.stats.mana as i32,
        victory: false,
        defeat: false,
        experience_gained: None,
        victory_message: None,
        defeat_message: None,
        level_up: None,
        game_state: Some(game_state),
        enemy_attacks,
    }))
}
