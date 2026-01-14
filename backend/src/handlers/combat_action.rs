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

/// Combat action request
#[derive(Debug, Deserialize)]
#[serde(tag = "actionType", rename_all = "camelCase")]
pub enum CombatActionRequest {
    #[serde(rename = "melee")]
    Melee,
    #[serde(rename = "ability")]
    Ability {
        #[serde(rename = "abilityId")]
        ability_id: String,
    },
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CombatActionResult {
    pub attacks: Vec<SingleAttack>,
    pub total_damage: i32,
    pub enemy_health: i32,
    pub enemy_attacks: Vec<SingleAttack>,
    pub player_health: i32,
    pub player_mana: i32,
    pub victory: bool,
    pub defeat: bool,
    pub experience_gained: Option<i32>,
    pub victory_message: Option<String>,
    pub defeat_message: Option<String>,
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

/// Parse attack description template, replacing placeholders with actual values
fn parse_attack_description(template: &str, damage: i32, name: &str) -> String {
    template
        .replace("${damage}", &damage.to_string())
        .replace("${x}", &damage.to_string())
        .replace("${name}", name)
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
        .map_err(|e| AppError::from(e))?;

    let mut abilities = Vec::new();
    for char_ability in character_abilities {
        if let Ok(ability) = repo.get_ability(&char_ability.ability_id).await {
            abilities.push(ability);
        }
    }

    // === ACTION-SPECIFIC LOGIC: Calculate player attacks ===
    let mut attacks = Vec::new();
    let mut mana_cost = 0i64;

    match action {
        CombatActionRequest::Melee => {
            // Check for dual wield passive
            let has_dual_wield = abilities.iter().any(|ability| {
                ability.ability_type == "passive"
                    && ability.passive_effect.as_deref() == Some("dual_wield")
            });

            // First attack (main hand)
            let damage = DamageCalculator::calculate_melee_attack(character.stats.might);
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
                let damage = DamageCalculator::calculate_melee_attack(character.stats.might);
                attacks.push(SingleAttack {
                    damage,
                    description: format!(
                        "Your off-hand weapon strikes true, dealing **{}** damage!",
                        damage
                    ),
                    is_dual_wield: true,
                });
            }
        }
        CombatActionRequest::Ability { ability_id } => {
            // Validate character owns this ability
            let ability = abilities
                .iter()
                .find(|a| a.id == ability_id)
                .ok_or_else(|| AppError::validation_error("Ability not found or not unlocked"))?;

            // Check mana cost
            if character.stats.mana < ability.mana_cost {
                return Err(AppError::validation_error(&format!(
                    "Not enough mana. Required: {}, Available: {}",
                    ability.mana_cost, character.stats.mana
                )));
            }

            // Deduct mana cost
            mana_cost = ability.mana_cost;
            character.stats.mana -= mana_cost;

            // Calculate damage from formula
            let damage = if ability.damage_formula.is_some() {
                DamageCalculator::calculate_damage(ability, &character, None).map_err(|e| {
                    AppError::validation_error(&format!("Damage calculation failed: {}", e))
                })?
            } else {
                // Fallback to might-based if no formula
                DamageCalculator::calculate_melee_attack(character.stats.might)
            };

            // Use ability's attack description template
            let description = if let Some(ref template) = ability.attack_description {
                parse_attack_description(template, damage, &ability.name)
            } else {
                format!("You use {} for **{}** damage!", ability.name, damage)
            };

            attacks.push(SingleAttack {
                damage,
                description,
                is_dual_wield: false,
            });
        }
    }

    let total_damage: i32 = attacks.iter().map(|a| a.damage).sum();

    // === SHARED COMBAT FLOW ===

    // Increment turn number
    let current_turn = game_state
        .get("turnNumber")
        .and_then(|v| v.as_i64())
        .unwrap_or(1);
    game_state["turnNumber"] = serde_json::json!(current_turn + 1);

    // Update enemy health
    let enemy = game_state
        .get_mut("enemy")
        .ok_or_else(|| AppError::validation_error("No enemy in game state"))?;

    let current_enemy_health = enemy
        .get("health")
        .and_then(|v| v.as_i64())
        .ok_or_else(|| AppError::validation_error("Invalid enemy health"))?
        as i32;

    let new_enemy_health = (current_enemy_health - total_damage).max(0);
    enemy["health"] = serde_json::json!(new_enemy_health);

    // Get enemy details before we potentially modify game_state further
    let enemy_name = enemy
        .get("name")
        .and_then(|v| v.as_str())
        .unwrap_or("enemy")
        .to_string();
    let enemy_exp_reward = enemy
        .get("experienceReward")
        .and_then(|v| v.as_i64())
        .unwrap_or(50) as i32;
    let enemy_might = enemy
        .get("stats")
        .and_then(|s| s.get("might"))
        .and_then(|v| v.as_i64())
        .unwrap_or(10);
    let enemy_attack_template = enemy
        .get("attackDescription")
        .and_then(|v| v.as_str())
        .map(|s| s.to_string());

    // Check for victory
    let victory = new_enemy_health <= 0;
    let mut experience_gained = None;
    let mut victory_message = None;
    let mut level_up = None;
    let mut enemy_attacks = Vec::new();
    let mut player_health = game_state
        .get("playerHealth")
        .and_then(|v| v.as_i64())
        .unwrap_or(character.stats.health) as i32;
    let mut defeat = false;
    let mut defeat_message = None;

    if victory {
        // Award experience
        experience_gained = Some(enemy_exp_reward);

        // Calculate total accumulated experience
        let mut total_accumulated_exp = 0;
        for lvl in 2..=character.level {
            total_accumulated_exp += experience_for_level(lvl);
        }
        total_accumulated_exp += character.experience;
        let new_total_experience = total_accumulated_exp + enemy_exp_reward as i64;

        // Check for level up
        let old_level = character.level;
        let (new_level, exp_into_level, exp_for_next) =
            calculate_level_from_experience(new_total_experience);

        // Update experience
        repo.update_character_experience_progress(
            &character_id,
            &claims.sub,
            exp_into_level,
            exp_for_next,
        )
        .await
        .map_err(|e| AppError::from(e))?;

        // Handle level up
        if new_level > old_level {
            repo.update_character_level(&character_id, &claims.sub, new_level)
                .await
                .map_err(|e| AppError::from(e))?;

            let (might, defense, magic, resistance, agility, max_health, max_mana) =
                calculate_stat_increases_for_level(&character.class_id, new_level);

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

            // Unlock new abilities
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

            // Fully restore health and mana on level up
            character.stats.health = character.stats.max_health + max_health;
            character.stats.mana = character.stats.max_mana + max_mana;
        }

        // Update health and mana in database
        repo.update_character_health(&character_id, &claims.sub, character.stats.health)
            .await
            .map_err(|e| AppError::from(e))?;
        repo.update_character_mana(&character_id, &claims.sub, character.stats.mana)
            .await
            .map_err(|e| AppError::from(e))?;

        // Deduct adventure
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
            enemy_name, enemy_exp_reward
        ));

        tracing::info!(
            "Character {} defeated {} and gained {} experience",
            character_id,
            enemy_name,
            enemy_exp_reward
        );
    } else {
        // Enemy counterattacks
        let counter_damage = DamageCalculator::calculate_melee_attack(enemy_might);

        let attack_description = if let Some(template) = enemy_attack_template {
            parse_attack_description(&template, counter_damage, &enemy_name)
        } else {
            format!(
                "{} counterattacks, dealing **{}** damage!",
                enemy_name, counter_damage
            )
        };

        enemy_attacks.push(SingleAttack {
            damage: counter_damage,
            description: attack_description,
            is_dual_wield: false,
        });

        // Update player health
        player_health = (player_health - counter_damage).max(0);
        game_state["playerHealth"] = serde_json::json!(player_health);
        character.stats.health = player_health as i64;

        // Update character health in database
        repo.update_character_health(&character_id, &claims.sub, character.stats.health)
            .await
            .map_err(|e| AppError::from(e))?;

        // Check for defeat
        if player_health <= 0 {
            defeat = true;
            defeat_message = Some(format!(
                "You have been defeated by {}! You lose 1 adventure and gain no rewards.",
                enemy_name
            ));

            // Deduct adventure
            let new_adventures = (character.stats.adventures - 1).max(0);
            repo.update_character_adventures_with_user(&character_id, &claims.sub, new_adventures)
                .await
                .map_err(|e| AppError::from(e))?;

            // Clear game state (combat is over)
            repo.update_character_game_state(&character_id, &claims.sub, None)
                .await
                .map_err(|e| AppError::from(e))?;

            tracing::info!("Character {} was defeated by {}", character_id, enemy_name);
        }
    }

    // Update mana if it was consumed (ability use)
    if mana_cost > 0 && !victory && !defeat {
        repo.update_character_mana(&character_id, &claims.sub, character.stats.mana)
            .await
            .map_err(|e| AppError::from(e))?;
    }

    // Prepare game state for response
    let mut response_game_state = game_state.clone();

    // Mark combat as finished when it ends
    if victory || defeat {
        response_game_state["finished"] = serde_json::json!(true);
    }

    // Save updated game state if combat continues
    if !victory && !defeat {
        let updated_game_state_str = serde_json::to_string(&game_state).map_err(|e| {
            AppError::validation_error(&format!("Failed to serialize game state: {}", e))
        })?;

        repo.update_character_game_state(&character_id, &claims.sub, Some(updated_game_state_str))
            .await
            .map_err(|e| AppError::from(e))?;

        tracing::info!(
            "Character {} performed combat action, enemy health: {} -> {}",
            character_id,
            current_enemy_health,
            new_enemy_health
        );
    }

    Ok(Json(CombatActionResult {
        attacks,
        total_damage,
        enemy_health: new_enemy_health,
        enemy_attacks,
        player_health,
        player_mana: character.stats.mana as i32,
        victory,
        defeat,
        experience_gained,
        victory_message,
        defeat_message,
        level_up,
        game_state: Some(response_game_state),
    }))
}
