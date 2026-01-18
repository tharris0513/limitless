use crate::error::AppError;
use crate::models::{Ability, Character, PassiveEffectType};

// Game state field name constants
pub const GS_ENEMY: &str = "enemy";
pub const GS_ENEMY_HEALTH: &str = "health";
pub const GS_ENEMY_NAME: &str = "name";
pub const GS_ENEMY_STATS: &str = "stats";
pub const GS_ENEMY_MIGHT: &str = "might";
pub const GS_ENEMY_EXP_REWARD: &str = "experienceReward";
pub const GS_ENEMY_ATTACK_DESC: &str = "attackDescription";
pub const GS_PRIMARY_WEAPON_ENCHANTED: &str = "primaryWeaponEnchanted";
pub const GS_SECONDARY_WEAPON_ENCHANTED: &str = "secondaryWeaponEnchanted";
pub const GS_ABILITY_COOLDOWNS: &str = "abilityCooldowns";
pub const GS_TURN_NUMBER: &str = "turnNumber";
pub const GS_PLAYER_HEALTH: &str = "playerHealth";
pub const GS_FINISHED: &str = "finished";

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
    #[allow(dead_code)]
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
                    if effect.effect_type == crate::models::EffectType::Passive {
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
    #[allow(dead_code)]
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

/// Parse attack description template, replacing placeholders with actual values
pub fn parse_attack_description(template: &str, damage: i32, name: &str) -> String {
    template
        .replace("${damage}", &damage.to_string())
        .replace("${x}", &damage.to_string())
        .replace("${name}", name)
}

/// Parse enchantment type from game state string
pub fn parse_enchantment_from_state(enchant_str: &str) -> Option<PassiveEffectType> {
    match enchant_str {
        "fire" => Some(PassiveEffectType::EnchantWeaponFire),
        "frost" => Some(PassiveEffectType::EnchantWeaponFrost),
        "lightning" => Some(PassiveEffectType::EnchantWeaponLightning),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::{AbilityEffect, ActiveEffectType, CharacterStats, EffectType};

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
            stats: CharacterStats {
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
            },
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

    #[test]
    fn test_ability_processor_has_passive() {
        let character = create_test_character();

        // Ability with dual-wield passive
        let dual_wield_ability = Ability {
            id: "dual_wield".to_string(),
            name: "Dual Wield".to_string(),
            description: "Wield two weapons".to_string(),
            ability_type: "passive".to_string(),
            mana_cost: None,
            cooldown: None,
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
            }],
        };

        let abilities = vec![dual_wield_ability];
        let processor = AbilityProcessor::new(&character, &abilities);

        assert!(processor.has_passive(PassiveEffectType::DualWield));
        assert!(!processor.has_passive(PassiveEffectType::EnchantWeapon));
    }

    #[test]
    fn test_parse_attack_description() {
        let template = "You strike ${name} for ${damage} damage!";
        let result = parse_attack_description(template, 42, "Goblin");

        assert_eq!(result, "You strike Goblin for 42 damage!");
    }

    #[test]
    fn test_parse_attack_description_with_x_placeholder() {
        let template = "You deal ${x} damage to the ${name}!";
        let result = parse_attack_description(template, 25, "Dragon");

        assert_eq!(result, "You deal 25 damage to the Dragon!");
    }

    #[test]
    fn test_game_state_helper_accessors() {
        let game_state = serde_json::json!({
            "primaryWeaponEnchanted": "fire",
            "secondaryWeaponEnchanted": "frost",
            "abilityCooldowns": {
                "fireball": 2
            },
            "turnNumber": 5,
            "playerHealth": 80
        });

        let helper = GameStateHelper::new(&game_state);

        assert_eq!(helper.primary_enchantment_str(), "fire");
        assert_eq!(helper.secondary_enchantment_str(), "frost");
        assert_eq!(helper.turn_number(), 5);
        assert_eq!(helper.player_health(100), 80);

        let cooldowns = helper.ability_cooldowns();
        assert_eq!(cooldowns.get("fireball"), Some(&2));
    }

    #[test]
    fn test_enemy_helper_accessors() {
        let enemy = serde_json::json!({
            "name": "Orc Warrior",
            "health": 120,
            "experienceReward": 200,
            "stats": {
                "might": 15
            },
            "attackDescription": "${name} swings for ${damage}!"
        });

        let helper = EnemyHelper::new(&enemy);

        assert_eq!(helper.name(), "Orc Warrior");
        assert_eq!(helper.health().unwrap(), 120);
        assert_eq!(helper.exp_reward(), 200);
        assert_eq!(helper.might(), 15);
        assert_eq!(
            helper.attack_description().unwrap(),
            "${name} swings for ${damage}!"
        );
    }

    #[test]
    fn test_parse_enchantment_from_state() {
        assert_eq!(
            parse_enchantment_from_state("fire"),
            Some(PassiveEffectType::EnchantWeaponFire)
        );
        assert_eq!(
            parse_enchantment_from_state("frost"),
            Some(PassiveEffectType::EnchantWeaponFrost)
        );
        assert_eq!(
            parse_enchantment_from_state("lightning"),
            Some(PassiveEffectType::EnchantWeaponLightning)
        );
        assert_eq!(parse_enchantment_from_state("invalid"), None);
    }

    #[test]
    fn test_ability_processor_calculate_enchantment_damage() {
        let mut character = create_test_character();
        character.stats.magic = 20;

        let abilities = vec![];
        let processor = AbilityProcessor::new(&character, &abilities);

        // Formula: magic * 0.5
        let damage = processor.calculate_enchantment_damage();
        assert_eq!(damage, 10); // 20 * 0.5 = 10
    }

    #[test]
    fn test_ability_processor_get_weapon_enchantment_type() {
        let character = create_test_character();

        let fire_enchant = Ability {
            id: "fire_enchant".to_string(),
            name: "Fire Enchantment".to_string(),
            description: "Add fire damage".to_string(),
            ability_type: "passive".to_string(),
            mana_cost: None,
            cooldown: None,
            damage_formula: None,
            heal_formula: None,
            effect_formula: None,
            passive_effect: None,
            attack_description: None,
            effects: vec![AbilityEffect {
                id: "fire_ench".to_string(),
                effect_type: EffectType::Passive,
                active_type: None,
                passive_type: Some(PassiveEffectType::EnchantWeaponFire),
                formula: None,
                attack_description: None,
            }],
        };

        let abilities = vec![fire_enchant];
        let processor = AbilityProcessor::new(&character, &abilities);

        assert_eq!(
            processor.get_weapon_enchantment_type(),
            Some(PassiveEffectType::EnchantWeaponFire)
        );
    }
}
