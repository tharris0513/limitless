pub mod helpers;

use crate::damage_calculator::DamageCalculator;
use crate::error::AppError;
use crate::level_system::{
    calculate_level_from_experience, calculate_stat_increases_for_level, experience_for_level,
};
use crate::models::{
    Ability, ActiveEffectType, Character, CreatureInCombat, EffectType, PassiveEffectType,
};
use crate::repository::UserRepository;
use serde::{Deserialize, Serialize};

pub use helpers::{
    parse_attack_description, AbilityProcessor, GameStateHelper, GS_ABILITY_COOLDOWNS, GS_ENEMY,
    GS_PLAYER_HEALTH, GS_PRIMARY_WEAPON_ENCHANTED, GS_SECONDARY_WEAPON_ENCHANTED, GS_STATUS,
    GS_TURN_NUMBER,
};

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
    pub total_damage: i64,
    pub enemy_health: i64,
    pub enemy_attacks: Vec<SingleAttack>,
    pub player_health: i64,
    pub player_mana: i64,
    pub victory: bool,
    pub defeat: bool,
    pub experience_gained: Option<i64>,
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
    pub damage: i64,
    pub description: String,
    pub is_dual_wield: bool,
}

/// Apply weapon enchantment to attacks if available
/// Returns Some(damage) if enchantment was applied, None otherwise
/// Important: This should only be called once per attack in the sequence to avoid double-application
pub fn apply_weapon_enchantment(
    processor: &AbilityProcessor,
    attacks: &mut Vec<SingleAttack>,
    enchantment_type: Option<PassiveEffectType>,
    _weapon: &str, // "primary" or "secondary"
) -> Option<i64> {
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
pub fn process_melee_attacks(
    processor: &AbilityProcessor,
    character: &Character,
    game_state: &serde_json::Value,
    enemy_defense: i64,
) -> Vec<SingleAttack> {
    let mut attacks = Vec::new();
    let has_dual_wield = processor.has_passive(PassiveEffectType::DualWield);

    // First attack (main hand)
    let damage = DamageCalculator::calculate_melee_attack(character.might, enemy_defense);
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
        let damage = DamageCalculator::calculate_melee_attack(character.might, enemy_defense);
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
pub fn process_ability_attacks(
    ability: &Ability,
    character: &Character,
    processor: &AbilityProcessor,
    game_state: &mut serde_json::Value,
) -> Result<Vec<SingleAttack>, AppError> {
    let mut attacks = Vec::new();

    // Get enemy Creature object from gate state
    let enemy: CreatureInCombat = serde_json::from_value(
        game_state
            .get(GS_ENEMY)
            .ok_or_else(|| AppError::validation_error("No enemy in game state"))?
            .clone(),
    )
    .map_err(|e| {
        AppError::validation_error(&format!("Failed to parse enemy from game state: {}", e))
    })?;
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
                                let damage = DamageCalculator::test_formula_with_enemy(
                                    formula, &character, &enemy,
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
            // Fallback to might-based if no formula (won't apply defense here as this is legacy)
            DamageCalculator::calculate_melee_attack(character.might, 0)
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
pub fn update_cooldowns(
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
pub async fn handle_victory(
    repo: &UserRepository,
    character_id: &str,
    user_id: &str,
    character: &mut Character,
    enemy_name: String,
    enemy_exp_reward: i64,
) -> Result<(Option<i64>, Option<String>, Option<LevelUpInfo>), AppError> {
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
        character.health = character.max_health + max_health;
        character.mana = character.max_mana + max_mana;
    }

    // Update health and mana in database
    repo.update_character_health(character_id, user_id, character.health)
        .await
        .map_err(AppError::from)?;
    repo.update_character_mana(character_id, user_id, character.mana)
        .await
        .map_err(AppError::from)?;

    // Deduct adventure
    let new_adventures = (character.adventures - 1).max(0);
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
pub async fn handle_defeat(
    repo: &UserRepository,
    character_id: &str,
    user_id: &str,
    character: &Character,
    enemy_name: &String,
) -> Result<Option<String>, AppError> {
    // Deduct adventure
    let new_adventures = (character.adventures - 1).max(0);
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

/// Apply enemy counterattack with player defense reduction
pub fn apply_enemy_attack(
    enemy_might: i64,
    player_defense: i64,
    enemy_name: &str,
    enemy_attack_template: String,
) -> (Vec<SingleAttack>, i64) {
    // Enemy melee attack reduced by player defense
    let mitigated_damage = DamageCalculator::calculate_melee_attack(enemy_might, player_defense);

    tracing::debug!(
        "Enemy attack: enemy_might={}, player_defense={}, final={}",
        enemy_might,
        player_defense,
        mitigated_damage
    );

    let attack_description = if !enemy_attack_template.is_empty() {
        parse_attack_description(&enemy_attack_template, mitigated_damage, enemy_name)
    } else {
        format!(
            "{} counterattacks, dealing **{}** damage!",
            enemy_name, mitigated_damage
        )
    };

    let attacks = vec![SingleAttack {
        damage: mitigated_damage,
        description: attack_description,
        is_dual_wield: false,
    }];

    (attacks, mitigated_damage)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::{AbilityEffect, Character};

    /// Helper to create a basic test character
    fn create_test_character() -> Character {
        Character {
            id: "char_test".to_string(),
            user_id: "user_test".to_string(),
            name: "Test Hero".to_string(),
            class_id: "warrior".to_string(),
            level: 5,
            experience: 100,
            experience_to_next: 500,
            might: 15,
            defense: 10,
            magic: 8,
            resistance: 6,
            agility: 12,
            health: 100,
            max_health: 100,
            mana: 50,
            max_mana: 50,
            adventures: 10,
            active_buffs: Some(vec![].into()),
            location: "town".to_string(),
            created_at: "2026-01-01T00:00:00Z".to_string(),
            last_played: "2026-01-01T00:00:00Z".to_string(),
            game_state: Some(
                serde_json::json!({
                    "enemy": {
                        "name": "Test Goblin",
                        "health": 50,
                        "stats": {
                            "might": 10
                        },
                        "experienceReward": 100
                    },
                    "playerHealth": 100,
                    "turnNumber": 1
                })
                .to_string(),
            ),
        }
    }

    /// Helper to create a basic damage ability
    fn create_damage_ability() -> Ability {
        Ability {
            id: "fireball".to_string(),
            name: "Fireball".to_string(),
            description: "A ball of fire".to_string(),
            ability_type: "combat".to_string(),
            mana_cost: Some(10),
            cooldown: Some(2),
            duration: None,
            damage_formula: Some("(magic * 1.5) + 10".to_string()),
            heal_formula: None,
            effect_formula: None,
            passive_effect: None,
            attack_description: Some("You hurl a fireball for ${damage} damage!".to_string()),
            effects: vec![AbilityEffect {
                id: "fireball_damage".to_string(),
                effect_type: EffectType::Active,
                active_type: Some(ActiveEffectType::Damage),
                passive_type: None,
                formula: Some("(magic * 1.5) + 10".to_string()),
                attack_description: Some("You hurl a fireball for ${damage} damage!".to_string()),
                passive_mode: None,
                stat_modifier: None,
            }],
        }
    }

    /// Helper to create a basic test enemy
    fn create_test_enemy() -> serde_json::Value {
        serde_json::json!({
            "name": "Goblin",
            "health": 50,
            "stats": {
                "might": 10,
                "defense": 5,
            },
            "experienceReward": 100
        })
    }

    #[test]
    fn test_process_melee_attacks_basic() {
        let character = create_test_character();
        let enemy = create_test_enemy();
        let abilities = vec![];
        let processor = AbilityProcessor::new(&character, &abilities);
        let game_state = serde_json::json!({
            "enemy": {
                "name": "Goblin",
                "health": 50
            },
            "turnNumber": 1
        });

        let attacks = process_melee_attacks(
            &processor,
            &character,
            &game_state,
            enemy["stats"]["defense"].as_i64().unwrap(),
        );

        // Should have exactly 1 attack (no dual-wield)
        assert_eq!(attacks.len(), 1);
        assert!(!attacks[0].is_dual_wield);
        assert!(attacks[0].damage > 0);
    }

    #[test]
    fn test_process_melee_attacks_with_dual_wield() {
        let character = create_test_character();

        let dual_wield_ability = Ability {
            id: "dual_wield".to_string(),
            name: "Dual Wield".to_string(),
            description: "Wield two weapons".to_string(),
            ability_type: "passive".to_string(),
            mana_cost: None,
            cooldown: None,
            duration: None,
            damage_formula: None,
            heal_formula: None,
            effect_formula: None,
            passive_effect: None,
            attack_description: None,
            effects: vec![AbilityEffect {
                id: "dual_wield_passive".to_string(),
                effect_type: EffectType::Passive,
                active_type: None,
                passive_type: Some(PassiveEffectType::DualWield),
                formula: None,
                attack_description: None,
                passive_mode: None,
                stat_modifier: None,
            }],
        };

        let abilities = vec![dual_wield_ability];
        let processor = AbilityProcessor::new(&character, &abilities);
        let game_state = serde_json::json!({
            "enemy": {
                "name": "Goblin",
                "health": 50
            },
            "turnNumber": 1
        });

        let attacks = process_melee_attacks(&processor, &character, &game_state, 5);

        // Should have 2 attacks (main hand + off-hand)
        assert_eq!(attacks.len(), 2);
        assert!(!attacks[0].is_dual_wield); // First attack is main hand
        assert!(attacks[1].is_dual_wield); // Second attack is off-hand
    }

    #[test]
    fn test_process_melee_attacks_with_enchantment() {
        let character = create_test_character();

        // Create enchant weapon passive
        let enchant_ability = Ability {
            id: "enchant_weapon".to_string(),
            name: "Enchant Weapon".to_string(),
            description: "Enchant your weapon".to_string(),
            ability_type: "passive".to_string(),
            mana_cost: None,
            cooldown: None,
            duration: None,
            damage_formula: None,
            heal_formula: None,
            effect_formula: None,
            passive_effect: None,
            attack_description: None,
            effects: vec![AbilityEffect {
                id: "enchant_passive".to_string(),
                effect_type: EffectType::Passive,
                active_type: None,
                passive_type: Some(PassiveEffectType::EnchantWeapon),
                formula: None,
                attack_description: None,
                passive_mode: None,
                stat_modifier: None,
            }],
        };

        // Create fire enchantment ability that provides damage
        let fire_enchant_ability = Ability {
            id: "fire_enchant".to_string(),
            name: "Fire Enchantment".to_string(),
            description: "Add fire damage".to_string(),
            ability_type: "passive".to_string(),
            mana_cost: None,
            cooldown: None,
            duration: None,
            damage_formula: None,
            heal_formula: None,
            effect_formula: None,
            passive_effect: None,
            attack_description: None,
            effects: vec![
                AbilityEffect {
                    id: "fire_enchant_passive".to_string(),
                    effect_type: EffectType::Passive,
                    active_type: None,
                    passive_type: Some(PassiveEffectType::EnchantWeaponFire),
                    formula: None,
                    attack_description: None,
                    passive_mode: None,
                    stat_modifier: None,
                },
                AbilityEffect {
                    id: "fire_enchant_damage".to_string(),
                    effect_type: EffectType::Active,
                    active_type: Some(ActiveEffectType::Damage),
                    passive_type: None,
                    formula: Some("magic * 0.5".to_string()),
                    attack_description: None,
                    passive_mode: None,
                    stat_modifier: None,
                },
            ],
        };

        let abilities = vec![enchant_ability, fire_enchant_ability];
        let processor = AbilityProcessor::new(&character, &abilities);
        let game_state = serde_json::json!({
            "enemy": {
                "name": "Goblin",
                "health": 50
            },
            "primaryWeaponEnchanted": "fire",
            "turnNumber": 1
        });

        let attacks = process_melee_attacks(&processor, &character, &game_state, 5);

        // Should have 2 attacks: melee + enchantment damage
        assert_eq!(attacks.len(), 2);
        assert!(attacks[0].description.contains("swing"));
        assert!(attacks[1].description.contains("fire"));
    }

    #[test]
    fn test_process_ability_attacks_damage_effect() {
        let mut character = create_test_character();
        character.magic = 20; // Set magic for damage calculation

        let ability = create_damage_ability();
        let abilities = vec![];
        let processor = AbilityProcessor::new(&character, &abilities);
        let enemey_in_combat = CreatureInCombat {
            name: "Goblin".to_string(),
            health: 50,
            max_health: 50,
            level: 3,
            introduction_text: "Intro".to_string(),
            might: 5,
            defense: 5,
            magic: 3,
            resistance: 3,
            agility: 3,
            creature_type: Some("humanoid".to_string()),
            experience_reward: 200,
            attack_description: "Rawr!".to_string(),
        };

        let mut game_state = serde_json::json!({
            "enemy": enemey_in_combat,
            "turnNumber": 1
        });

        let result = process_ability_attacks(&ability, &character, &processor, &mut game_state);

        assert!(result.is_ok(), "Ability processing failed: {:?}", result);
        let attacks = result.unwrap();
        assert_eq!(attacks.len(), 1);
        assert!(attacks[0].damage > 0);
        assert!(attacks[0].description.contains("fireball"));
    }

    #[test]
    fn test_update_cooldowns_decrements_existing() {
        let mut game_state = serde_json::json!({
            "abilityCooldowns": {
                "fireball": 2,
                "ice_blast": 1
            }
        });

        let action = CombatActionRequest::Melee;
        let abilities = vec![];

        let cooldowns = update_cooldowns(&mut game_state, &action, &abilities);

        // Fireball should decrement to 1
        assert_eq!(cooldowns.get("fireball"), Some(&1));
        // Ice blast should be removed (was 1, decremented to 0)
        assert_eq!(cooldowns.get("ice_blast"), None);
    }

    #[test]
    fn test_update_cooldowns_sets_new_cooldown() {
        let mut game_state = serde_json::json!({
            "abilityCooldowns": {}
        });

        let ability = create_damage_ability(); // Has cooldown of 2
        let abilities = vec![ability.clone()];
        let action = CombatActionRequest::Ability {
            ability_id: ability.id.clone(),
        };

        let cooldowns = update_cooldowns(&mut game_state, &action, &abilities);

        // Fireball should now be on cooldown for 2 turns
        assert_eq!(cooldowns.get("fireball"), Some(&2));
    }
}
