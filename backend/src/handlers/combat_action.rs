use crate::damage_calculator::DamageCalculator;
use crate::error::AppError;
use crate::level_system::{
    calculate_level_from_experience, calculate_stat_increases_for_level, experience_for_level,
};
use crate::middleware::AuthClaims;
use crate::models::{
    Ability, ActiveEffectType, Character, CharacterAbility, EffectType, PassiveEffectType,
};
use crate::repository::UserRepository;
use axum::{
    extract::{Path, State},
    Json,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

// Game state field name constants
const GS_ENEMY: &str = "enemy";
const GS_ENEMY_HEALTH: &str = "health";
const GS_ENEMY_NAME: &str = "name";
const GS_ENEMY_STATS: &str = "stats";
const GS_ENEMY_MIGHT: &str = "might";
const GS_ENEMY_EXP_REWARD: &str = "experienceReward";
const GS_ENEMY_ATTACK_DESC: &str = "attackDescription";
const GS_PRIMARY_WEAPON_ENCHANTED: &str = "primaryWeaponEnchanted";
const GS_SECONDARY_WEAPON_ENCHANTED: &str = "secondaryWeaponEnchanted";
const GS_ABILITY_COOLDOWNS: &str = "abilityCooldowns";
const GS_TURN_NUMBER: &str = "turnNumber";
const GS_PLAYER_HEALTH: &str = "playerHealth";
const GS_FINISHED: &str = "finished";

/// Helper struct to process abilities for a character
pub struct AbilityProcessor<'a> {
    character: &'a Character,
    abilities: &'a [Ability],
}

impl<'a> AbilityProcessor<'a> {
    pub fn new(character: &'a Character, abilities: &'a [Ability]) -> Self {
        Self {
            character,
            abilities,
        }
    }

    /// Check if character has a specific passive effect
    pub fn has_passive(&self, passive_type: PassiveEffectType) -> bool {
        self.abilities.iter().any(|a| a.has_passive(passive_type))
    }

    /// Get the current weapon enchantment type the character has
    /// Returns Some(PassiveEffectType::EnchantWeaponFire), etc., or None
    pub fn get_weapon_enchantment_type(&self) -> Option<PassiveEffectType> {
        for ability in self.abilities {
            if let Some(enchant_type) = ability.get_enchantment_type() {
                return Some(enchant_type);
            }
        }
        None
    }

    /// Calculate enchantment damage using standardized formula
    pub fn calculate_enchantment_damage(&self) -> i32 {
        (self.character.stats.magic as f64 * 0.5).round() as i32
    }

    /// Check if character has the base weapon enchantment passive
    pub fn has_base_weapon_enchant_passive(&self) -> bool {
        self.has_passive(PassiveEffectType::EnchantWeapon)
    }

    /// Get abilities that provide enchantment damage for a specific type
    pub fn get_enchantment_damage_abilities(
        &self,
        enchant_type: Option<PassiveEffectType>,
    ) -> Vec<&Ability> {
        self.abilities
            .iter()
            .filter(|ability| {
                // Check if ability has the matching passive enchantment effect
                let has_matching_passive = ability.effects.iter().any(|effect| {
                    if effect.effect_type == EffectType::Passive {
                        if let Some(passive_type) = &effect.passive_type {
                            if let Some(ref ench_type) = enchant_type {
                                // Match specific type
                                return passive_type == ench_type;
                            } else {
                                // No type specified, accept any enchant_weapon_* passive
                                return matches!(
                                    passive_type,
                                    PassiveEffectType::EnchantWeaponFire
                                        | PassiveEffectType::EnchantWeaponFrost
                                        | PassiveEffectType::EnchantWeaponLightning
                                );
                            }
                        }
                    }
                    false
                });

                // Check if ability provides enchantment damage
                has_matching_passive && ability.provides_enchantment_damage()
            })
            .collect()
    }

    /// Find ability by ID
    pub fn find_ability(&self, ability_id: &str) -> Option<&Ability> {
        self.abilities.iter().find(|a| a.id == ability_id)
    }
}

/// Helper to access game state fields with proper error handling
pub struct GameStateHelper<'a> {
    state: &'a serde_json::Value,
}

impl<'a> GameStateHelper<'a> {
    pub fn new(state: &'a serde_json::Value) -> Self {
        Self { state }
    }

    /// Get primary weapon enchantment type
    pub fn primary_enchantment(&self) -> Option<PassiveEffectType> {
        self.state
            .get(GS_PRIMARY_WEAPON_ENCHANTED)
            .and_then(|v| v.as_str())
            .and_then(parse_enchantment_from_state)
    }

    /// Get secondary weapon enchantment type
    pub fn secondary_enchantment(&self) -> Option<PassiveEffectType> {
        self.state
            .get(GS_SECONDARY_WEAPON_ENCHANTED)
            .and_then(|v| v.as_str())
            .and_then(parse_enchantment_from_state)
    }

    /// Get primary weapon enchantment as string
    pub fn primary_enchantment_str(&self) -> &str {
        self.state
            .get(GS_PRIMARY_WEAPON_ENCHANTED)
            .and_then(|v| v.as_str())
            .unwrap_or("")
    }

    /// Get secondary weapon enchantment as string
    pub fn secondary_enchantment_str(&self) -> &str {
        self.state
            .get(GS_SECONDARY_WEAPON_ENCHANTED)
            .and_then(|v| v.as_str())
            .unwrap_or("")
    }

    /// Get ability cooldowns map
    pub fn ability_cooldowns(&self) -> std::collections::HashMap<String, i32> {
        self.state
            .get(GS_ABILITY_COOLDOWNS)
            .and_then(|v| serde_json::from_value(v.clone()).ok())
            .unwrap_or_default()
    }

    /// Get current turn number
    pub fn turn_number(&self) -> i64 {
        self.state
            .get(GS_TURN_NUMBER)
            .and_then(|v| v.as_i64())
            .unwrap_or(1)
    }

    /// Get player health
    pub fn player_health(&self, default: i64) -> i32 {
        self.state
            .get(GS_PLAYER_HEALTH)
            .and_then(|v| v.as_i64())
            .unwrap_or(default) as i32
    }

    /// Get enemy object
    pub fn enemy(&self) -> Result<&serde_json::Value, AppError> {
        self.state
            .get(GS_ENEMY)
            .ok_or_else(|| AppError::validation_error("No enemy in game state"))
    }
}

/// Helper to extract enemy details from enemy JSON object
pub struct EnemyHelper<'a> {
    enemy: &'a serde_json::Value,
}

impl<'a> EnemyHelper<'a> {
    pub fn new(enemy: &'a serde_json::Value) -> Self {
        Self { enemy }
    }

    /// Get enemy health
    pub fn health(&self) -> Result<i32, AppError> {
        self.enemy
            .get(GS_ENEMY_HEALTH)
            .and_then(|v| v.as_i64())
            .ok_or_else(|| AppError::validation_error("Invalid enemy health"))
            .map(|h| h as i32)
    }

    /// Get enemy name
    pub fn name(&self) -> String {
        self.enemy
            .get(GS_ENEMY_NAME)
            .and_then(|v| v.as_str())
            .unwrap_or("enemy")
            .to_string()
    }

    /// Get enemy experience reward
    pub fn exp_reward(&self) -> i32 {
        self.enemy
            .get(GS_ENEMY_EXP_REWARD)
            .and_then(|v| v.as_i64())
            .unwrap_or(50) as i32
    }

    /// Get enemy might stat
    pub fn might(&self) -> i64 {
        self.enemy
            .get(GS_ENEMY_STATS)
            .and_then(|s| s.get(GS_ENEMY_MIGHT))
            .and_then(|v| v.as_i64())
            .unwrap_or(10)
    }

    /// Get enemy attack description template
    pub fn attack_description(&self) -> Option<String> {
        self.enemy
            .get(GS_ENEMY_ATTACK_DESC)
            .and_then(|v| v.as_str())
            .map(|s| s.to_string())
    }
}

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
    pub ability_cooldowns: std::collections::HashMap<String, i32>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LevelUpInfo {
    pub new_level: i64,
    pub stat_increases: StatIncreases,
    pub abilities_learned: Vec<AbilityLearned>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AbilityLearned {
    pub id: String,
    pub name: String,
    pub description: String,
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

/// Parse enchantment type from game state string
fn parse_enchantment_from_state(enchant_str: &str) -> Option<PassiveEffectType> {
    match enchant_str {
        "fire" => Some(PassiveEffectType::EnchantWeaponFire),
        "frost" => Some(PassiveEffectType::EnchantWeaponFrost),
        "lightning" => Some(PassiveEffectType::EnchantWeaponLightning),
        _ => None,
    }
}

/// Apply weapon enchantment to attacks if available
/// Returns Some(damage) if enchantment was applied, None otherwise
/// Important: This should only be called once per attack in the sequence to avoid double-application
fn apply_weapon_enchantment(
    processor: &AbilityProcessor,
    attacks: &mut Vec<SingleAttack>,
    enchantment_type: Option<PassiveEffectType>,
    _weapon: &str, // "primary" or "secondary"
) -> Option<i32> {
    // Find abilities that provide enchantment damage for this type
    let enchant_abilities = processor.get_enchantment_damage_abilities(enchantment_type);

    // Only apply enchantment if we found matching abilities
    if enchant_abilities.is_empty() {
        return None;
    }

    // Calculate enchantment damage using standardized formula
    let enchant_damage = processor.calculate_enchantment_damage();

    // Get element name for message
    let element_name = if let Some(ref ench_type) = enchantment_type {
        ench_type.enchantment_element().unwrap_or("elemental")
    } else {
        "elemental"
    };

    // Create enchantment message
    let description = format!(
        "Your weapon does an extra **{}** {} damage!",
        enchant_damage, element_name
    );

    // Add enchantment attack
    attacks.push(SingleAttack {
        damage: enchant_damage,
        description,
        is_dual_wield: false,
    });

    Some(enchant_damage)
}

/// Process melee attacks with dual-wield and enchantments
fn process_melee_attacks(
    processor: &AbilityProcessor,
    character: &Character,
    game_state: &serde_json::Value,
) -> Vec<SingleAttack> {
    let mut attacks = Vec::new();
    let has_dual_wield = processor.has_passive(PassiveEffectType::DualWield);

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

    // Apply primary weapon enchantment if active
    let gs_helper = GameStateHelper::new(game_state);
    let primary_enchanted = gs_helper.primary_enchantment();

    if let Some(enchant_type) = primary_enchanted {
        apply_weapon_enchantment(processor, &mut attacks, Some(enchant_type), "primary");
    }

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

        // Apply secondary weapon enchantment if active
        let secondary_enchanted = gs_helper.secondary_enchantment();

        if let Some(enchant_type) = secondary_enchanted {
            apply_weapon_enchantment(processor, &mut attacks, Some(enchant_type), "secondary");
        }
    }

    attacks
}

/// Process ability effects and generate attacks
fn process_ability_attacks(
    ability: &Ability,
    character: &Character,
    processor: &AbilityProcessor,
    game_state: &mut serde_json::Value,
) -> Result<Vec<SingleAttack>, AppError> {
    let mut attacks = Vec::new();

    if !ability.effects.is_empty() {
        // Use new effects system
        for effect in &ability.effects {
            if effect.effect_type == EffectType::Active {
                // Only process active effects during combat
                if let Some(ref active_type) = effect.active_type {
                    match active_type {
                        ActiveEffectType::Damage => {
                            // Calculate damage from this effect's formula
                            if let Some(ref formula) = effect.formula {
                                let damage = DamageCalculator::test_formula(
                                    formula,
                                    &character.stats,
                                    character.level,
                                )
                                .map_err(|e| {
                                    AppError::validation_error(&format!(
                                        "Damage calculation failed for effect {}: {}",
                                        effect.id, e
                                    ))
                                })?;

                                // Use effect's attack description or create default
                                let description = if let Some(ref template) =
                                    effect.attack_description
                                {
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
                        ActiveEffectType::Heal => {
                            // TODO: Implement healing effects
                        }
                        ActiveEffectType::Buff | ActiveEffectType::Debuff => {
                            // TODO: Implement buff/debuff effects
                        }
                    }
                }
            } else if effect.effect_type == EffectType::Passive {
                // Check if this ability applies a weapon enchantment
                if let Some(ref passive_type) = effect.passive_type {
                    match passive_type {
                        PassiveEffectType::EnchantWeaponFire
                        | PassiveEffectType::EnchantWeaponFrost
                        | PassiveEffectType::EnchantWeaponLightning => {
                            // First, verify the character has the base "enchant_weapon" passive
                            if !processor.has_base_weapon_enchant_passive() {
                                continue;
                            }

                            // Get enchantment element name
                            let ench_type = passive_type.enchantment_element().unwrap();

                            // Check current enchantment status
                            let gs_helper = GameStateHelper::new(game_state);
                            let primary_enchanted = gs_helper.primary_enchantment_str();
                            let secondary_enchanted = gs_helper.secondary_enchantment_str();

                            // Check for dual wield
                            let has_dual_wield =
                                processor.has_passive(PassiveEffectType::DualWield);

                            // Apply enchantment
                            if primary_enchanted.is_empty() {
                                game_state[GS_PRIMARY_WEAPON_ENCHANTED] =
                                    serde_json::json!(ench_type);
                                attacks.push(SingleAttack {
                                    damage: 0,
                                    description: format!(
                                        "You enchant your primary weapon with the power of {}!",
                                        ench_type
                                    ),
                                    is_dual_wield: false,
                                });
                            } else if has_dual_wield && secondary_enchanted.is_empty() {
                                game_state[GS_SECONDARY_WEAPON_ENCHANTED] =
                                    serde_json::json!(ench_type);
                                attacks.push(SingleAttack {
                                    damage: 0,
                                    description: format!(
                                        "You enchant your secondary weapon with the power of {}!",
                                        ench_type
                                    ),
                                    is_dual_wield: false,
                                });
                            }
                        }
                        _ => {
                            // Other passive types not handled in combat actions
                        }
                    }
                }
            }
        }
    } else {
        // Fallback to legacy system if no effects defined
        let damage = if ability.damage_formula.is_some() {
            DamageCalculator::calculate_damage(ability, character, None).map_err(|e| {
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

    Ok(attacks)
}

/// Update ability cooldowns for the turn
fn update_cooldowns(
    game_state: &mut serde_json::Value,
    action: &CombatActionRequest,
    abilities: &[Ability],
) -> std::collections::HashMap<String, i32> {
    let gs_helper = GameStateHelper::new(game_state);
    let mut cooldowns = gs_helper.ability_cooldowns();

    // First, decrement all existing cooldowns (remove abilities at 0)
    cooldowns = cooldowns
        .into_iter()
        .filter_map(|(id, turns)| {
            let new_turns = (turns - 1).max(0);
            if new_turns > 0 {
                Some((id, new_turns))
            } else {
                None
            }
        })
        .collect();

    // Then, if an ability was used, set it on cooldown (AFTER decrementing)
    if let CombatActionRequest::Ability { ref ability_id } = action {
        if let Some(ability) = abilities.iter().find(|a| a.id == *ability_id) {
            if let Some(cooldown) = ability.cooldown {
                if cooldown > 0 {
                    cooldowns.insert(ability.id.clone(), cooldown as i32);
                }
            }
        }
    }

    game_state[GS_ABILITY_COOLDOWNS] = serde_json::json!(&cooldowns);
    cooldowns
}

/// Handle victory: award XP, check for level-up, update character
async fn handle_victory(
    repo: &UserRepository,
    character_id: &str,
    user_id: &str,
    character: &mut Character,
    enemy_name: String,
    enemy_exp_reward: i32,
) -> Result<(Option<i32>, Option<String>, Option<LevelUpInfo>), AppError> {
    let experience_gained = Some(enemy_exp_reward);

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
    repo.update_character_experience_progress(character_id, user_id, exp_into_level, exp_for_next)
        .await
        .map_err(AppError::from)?;

    let mut level_up = None;

    // Handle level up
    if new_level > old_level {
        repo.update_character_level(character_id, user_id, new_level)
            .await
            .map_err(AppError::from)?;

        let (might, defense, magic, resistance, agility, max_health, max_mana) =
            calculate_stat_increases_for_level(&character.class_id, new_level);

        repo.apply_stat_increases(
            character_id,
            user_id,
            might,
            defense,
            magic,
            resistance,
            agility,
            max_health,
            max_mana,
        )
        .await
        .map_err(AppError::from)?;

        // Unlock new abilities
        let abilities_learned = match repo
            .unlock_character_abilities(character_id, &character.class_id, new_level)
            .await
        {
            Ok(abilities) => abilities
                .into_iter()
                .map(|a| AbilityLearned {
                    id: a.id,
                    name: a.name,
                    description: a.description,
                })
                .collect(),
            Err(e) => {
                tracing::error!(
                    "Failed to unlock abilities for level {}: {:?}",
                    new_level,
                    e
                );
                Vec::new()
            }
        };

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
            abilities_learned,
        });

        // Fully restore health and mana on level up
        character.stats.health = character.stats.max_health + max_health;
        character.stats.mana = character.stats.max_mana + max_mana;
    }

    // Update health and mana in database
    repo.update_character_health(character_id, user_id, character.stats.health)
        .await
        .map_err(AppError::from)?;
    repo.update_character_mana(character_id, user_id, character.stats.mana)
        .await
        .map_err(AppError::from)?;

    // Deduct adventure
    let new_adventures = (character.stats.adventures - 1).max(0);
    repo.update_character_adventures_with_user(character_id, user_id, new_adventures)
        .await
        .map_err(AppError::from)?;

    // Clear game state (combat is over)
    repo.update_character_game_state(character_id, user_id, None)
        .await
        .map_err(AppError::from)?;

    let victory_message = Some(format!(
        "Victory! You have defeated {}! You gained **{}** experience.",
        enemy_name, enemy_exp_reward
    ));

    Ok((experience_gained, victory_message, level_up))
}

/// Handle defeat: deduct adventure, clear game state
async fn handle_defeat(
    repo: &UserRepository,
    character_id: &str,
    user_id: &str,
    character: &Character,
    enemy_name: String,
) -> Result<Option<String>, AppError> {
    // Deduct adventure
    let new_adventures = (character.stats.adventures - 1).max(0);
    repo.update_character_adventures_with_user(character_id, user_id, new_adventures)
        .await
        .map_err(AppError::from)?;

    // Clear game state (combat is over)
    repo.update_character_game_state(character_id, user_id, None)
        .await
        .map_err(AppError::from)?;

    let defeat_message = Some(format!(
        "You have been defeated by {}! You lose 1 adventure and gain no rewards.",
        enemy_name
    ));

    Ok(defeat_message)
}

/// Apply enemy counterattack
fn apply_enemy_attack(
    enemy_might: i64,
    enemy_name: &str,
    enemy_attack_template: Option<String>,
) -> (Vec<SingleAttack>, i32) {
    let counter_damage = DamageCalculator::calculate_melee_attack(enemy_might);

    let attack_description = if let Some(template) = enemy_attack_template {
        parse_attack_description(&template, counter_damage, enemy_name)
    } else {
        format!(
            "{} counterattacks, dealing **{}** damage!",
            enemy_name, counter_damage
        )
    };

    let attacks = vec![SingleAttack {
        damage: counter_damage,
        description: attack_description,
        is_dual_wield: false,
    }];

    (attacks, counter_damage)
}

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
    let _updated_character = repo
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

    // Update mana
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

    // === Generate player attacks based on action type ===
    let (attacks, mana_cost) = match action {
        CombatActionRequest::Melee => {
            let attacks = process_melee_attacks(&processor, &character, &game_state);
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
            if character.stats.mana < mana_cost_required {
                return Err(AppError::validation_error(&format!(
                    "Not enough mana. Required: {}, Available: {}",
                    mana_cost_required, character.stats.mana
                )));
            }

            // Process ability effects
            let attacks =
                process_ability_attacks(ability, &character, &processor, &mut game_state)?;

            (attacks, mana_cost_required)
        }
    };

    let total_damage: i32 = attacks.iter().map(|a| a.damage).sum();

    // === Apply combat round changes ===

    // Apply mana cost
    character.stats.mana -= mana_cost;

    // Increment turn number
    let gs_helper = GameStateHelper::new(&game_state);
    let current_turn = gs_helper.turn_number();
    game_state[GS_TURN_NUMBER] = serde_json::json!(current_turn + 1);

    // Update ability cooldowns
    let cooldowns = update_cooldowns(&mut game_state, &action, &abilities);

    // Update enemy health
    let enemy = game_state
        .get_mut(GS_ENEMY)
        .ok_or_else(|| AppError::validation_error("No enemy in game state"))?;

    let enemy_helper = EnemyHelper::new(enemy);
    let current_enemy_health = enemy_helper.health()?;

    let new_enemy_health = (current_enemy_health - total_damage).max(0);
    enemy[GS_ENEMY_HEALTH] = serde_json::json!(new_enemy_health);

    // Get enemy details before we potentially modify game_state further
    let enemy_helper = EnemyHelper::new(enemy);
    let enemy_name = enemy_helper.name();
    let enemy_exp_reward = enemy_helper.exp_reward();
    let enemy_might = enemy_helper.might();
    let enemy_attack_template = enemy_helper.attack_description();

    // === Handle combat outcome ===

    let victory = new_enemy_health <= 0;
    let gs_helper = GameStateHelper::new(&game_state);
    let mut player_health = gs_helper.player_health(character.stats.health);

    let (experience_gained, victory_message, level_up, enemy_attacks, defeat, defeat_message) =
        if victory {
            let (exp, vic_msg, lvl_up) = handle_victory(
                &repo,
                &character_id,
                &claims.sub,
                &mut character,
                enemy_name,
                enemy_exp_reward,
            )
            .await?;

            (exp, vic_msg, lvl_up, Vec::new(), false, None)
        } else {
            // Enemy counterattacks
            let (attacks, counter_damage) =
                apply_enemy_attack(enemy_might, &enemy_name, enemy_attack_template);

            // Update player health
            player_health = (player_health - counter_damage).max(0);
            game_state[GS_PLAYER_HEALTH] = serde_json::json!(player_health);
            character.stats.health = player_health as i64;

            // Update character health in database
            repo.update_character_health(&character_id, &claims.sub, character.stats.health)
                .await
                .map_err(AppError::from)?;

            // Check for defeat
            let (defeat, defeat_msg) = if player_health <= 0 {
                let msg = handle_defeat(&repo, &character_id, &claims.sub, &character, enemy_name)
                    .await?;
                (true, msg)
            } else {
                (false, None)
            };

            (None, None, None, attacks, defeat, defeat_msg)
        };

    // Update mana if it was consumed (ability use)
    if mana_cost > 0 && !victory && !defeat {
        repo.update_character_mana(&character_id, &claims.sub, character.stats.mana)
            .await
            .map_err(AppError::from)?;
    }

    // Prepare game state for response
    let mut response_game_state = game_state.clone();

    // Mark combat as finished when it ends
    if victory || defeat {
        response_game_state[GS_FINISHED] = serde_json::json!(true);
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
        player_mana: character.stats.mana as i32,
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
