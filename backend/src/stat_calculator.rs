use crate::models::{Character, ActiveBuff, Ability, AbilityEffect, EffectType};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Calculated stats for a character, including base stats plus all modifiers
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CalculatedStats {
    pub might: i64,
    pub defense: i64,
    pub magic: i64,
    pub resistance: i64,
    pub agility: i64,
    pub max_health: i64,
    pub max_mana: i64,
}

/// Type of stat modification
enum ModifierType {
    Set,
    Flat,
    Percentage,
}

/// A single stat modifier extracted from abilities or buffs
struct StatModifier {
    stat: String,
    value: f64,
    modifier_type: ModifierType,
}

impl StatModifier {
    fn from_ability_effect(effect: &AbilityEffect) -> Option<Self> {
        if effect.effect_type != EffectType::Passive {
            return None;
        }

        if effect.passive_mode.as_deref() != Some("stat_modifier") {
            return None;
        }

        let stat_mod = effect.stat_modifier.as_ref()?;

        let modifier_type = match stat_mod.modifier_type.as_str() {
            "set" => ModifierType::Set,
            "flat" => ModifierType::Flat,
            "percentage" => ModifierType::Percentage,
            _ => return None,
        };

        Some(StatModifier {
            stat: stat_mod.stat.clone(),
            value: stat_mod.value,
            modifier_type,
        })
    }
}

/// Calculate a single stat value with all modifiers applied
/// Formula: (base stat OR set stat + flat bonuses) * (1 + multiplicative bonuses)
fn calculate_stat(base_value: i64, modifiers: &[StatModifier]) -> i64 {
    // Step 1: Apply "set" modifiers (only the last one matters if multiple exist)
    let mut current_value = base_value as f64;
    for modifier in modifiers {
        if matches!(modifier.modifier_type, ModifierType::Set) {
            current_value = modifier.value;
        }
    }

    // Step 2: Apply flat modifiers
    for modifier in modifiers {
        if matches!(modifier.modifier_type, ModifierType::Flat) {
            current_value += modifier.value;
        }
    }

    // Step 3: Apply percentage modifiers
    let mut total_percentage: f64 = 1.0;
    for modifier in modifiers {
        if matches!(modifier.modifier_type, ModifierType::Percentage) {
            // Normalize the percentage value:
            // If value >= 1.0, treat as whole percentage (e.g., 20 = 20% = 0.20)
            // If value < 1.0, treat as decimal (e.g., 0.20 = 20%)
            let normalized_value = if modifier.value.abs() >= 1.0 {
                modifier.value / 100.0
            } else {
                modifier.value
            };
            total_percentage += normalized_value;
        }
    }
    current_value *= total_percentage;

    // Round to nearest integer and ensure non-negative
    current_value.round().max(0.0) as i64
}

/// Calculate all stats for a character given their abilities and active buffs
pub fn calculate_character_stats(
    character: &Character,
    abilities: &[Ability],
    active_buffs: Option<&Vec<ActiveBuff>>,
) -> CalculatedStats {
    // Collect all stat modifiers from passive abilities and active buffs
    let mut modifiers_by_stat: HashMap<String, Vec<StatModifier>> = HashMap::new();

    // Extract modifiers from passive abilities
    for ability in abilities {
        if ability.ability_type == "passive" {
            for effect in &ability.effects {
                if let Some(modifier) = StatModifier::from_ability_effect(effect) {
                    modifiers_by_stat
                        .entry(modifier.stat.clone())
                        .or_insert_with(Vec::new)
                        .push(modifier);
                }
            }
        }
    }

    // Extract modifiers from active buffs
    if let Some(buffs) = active_buffs {
        for buff in buffs {
            for effect in &buff.effects {
                if let Some(modifier) = StatModifier::from_ability_effect(effect) {
                    modifiers_by_stat
                        .entry(modifier.stat.clone())
                        .or_insert_with(Vec::new)
                        .push(modifier);
                }
            }
        }
    }

    // Calculate each stat with its modifiers
    CalculatedStats {
        might: calculate_stat(
            character.might,
            modifiers_by_stat.get("might").map(|v| v.as_slice()).unwrap_or(&[]),
        ),
        defense: calculate_stat(
            character.defense,
            modifiers_by_stat.get("defense").map(|v| v.as_slice()).unwrap_or(&[]),
        ),
        magic: calculate_stat(
            character.magic,
            modifiers_by_stat.get("magic").map(|v| v.as_slice()).unwrap_or(&[]),
        ),
        resistance: calculate_stat(
            character.resistance,
            modifiers_by_stat.get("resistance").map(|v| v.as_slice()).unwrap_or(&[]),
        ),
        agility: calculate_stat(
            character.agility,
            modifiers_by_stat.get("agility").map(|v| v.as_slice()).unwrap_or(&[]),
        ),
        max_health: calculate_stat(
            character.max_health,
            modifiers_by_stat.get("maxHealth").map(|v| v.as_slice()).unwrap_or(&[]),
        ),
        max_mana: calculate_stat(
            character.max_mana,
            modifiers_by_stat.get("maxMana").map(|v| v.as_slice()).unwrap_or(&[]),
        ),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::{AbilityEffect, PassiveEffectType};

    #[test]
    fn test_calculate_stat_no_modifiers() {
        let modifiers = vec![];
        assert_eq!(calculate_stat(10, &modifiers), 10);
    }

    #[test]
    fn test_calculate_stat_set_modifier() {
        let modifiers = vec![StatModifier {
            stat: "might".to_string(),
            value: 5.0,
            modifier_type: ModifierType::Set,
        }];
        assert_eq!(calculate_stat(10, &modifiers), 5);
    }

    #[test]
    fn test_calculate_stat_set_then_flat() {
        // A character has 10 might. They have a buff that sets their might to 5,
        // and another buff that adds +10 might. This should first set their might
        // to 5 then add 10 to give a total of 15 might.
        let modifiers = vec![
            StatModifier {
                stat: "might".to_string(),
                value: 5.0,
                modifier_type: ModifierType::Set,
            },
            StatModifier {
                stat: "might".to_string(),
                value: 10.0,
                modifier_type: ModifierType::Flat,
            },
        ];
        assert_eq!(calculate_stat(10, &modifiers), 15);
    }

    #[test]
    fn test_calculate_stat_flat_and_percentage() {
        // A character has 20 resistance. They have a buff that gives +10 to resistance,
        // a buff that gives 20%, and another buff that gives an additional 10%.
        // This should calculate as (20 + 10) * (1 + (.20 + .10)), for a total of 39 resistance.
        let modifiers = vec![
            StatModifier {
                stat: "resistance".to_string(),
                value: 10.0,
                modifier_type: ModifierType::Flat,
            },
            StatModifier {
                stat: "resistance".to_string(),
                value: 0.20,
                modifier_type: ModifierType::Percentage,
            },
            StatModifier {
                stat: "resistance".to_string(),
                value: 0.10,
                modifier_type: ModifierType::Percentage,
            },
        ];
        assert_eq!(calculate_stat(20, &modifiers), 39);
    }

    #[test]
    fn test_calculate_stat_set_and_percentage() {
        // A character has 10 agility. They have a buff that sets their agility to 0,
        // then a percentage buff of 50%. This should calculate to 0 * (1 + .5), or 0.
        let modifiers = vec![
            StatModifier {
                stat: "agility".to_string(),
                value: 0.0,
                modifier_type: ModifierType::Set,
            },
            StatModifier {
                stat: "agility".to_string(),
                value: 0.50,
                modifier_type: ModifierType::Percentage,
            },
        ];
        assert_eq!(calculate_stat(10, &modifiers), 0);
    }

    #[test]
    fn test_calculate_stat_flat_and_negative_percentage() {
        // A character has 10 defense. They have a buff that adds 10 defense,
        // and another buff that reduces defense by 20%.
        // This should calculate to (10 + 10) * (1 + -.20), or 16.
        let modifiers = vec![
            StatModifier {
                stat: "defense".to_string(),
                value: 10.0,
                modifier_type: ModifierType::Flat,
            },
            StatModifier {
                stat: "defense".to_string(),
                value: -0.20,
                modifier_type: ModifierType::Percentage,
            },
        ];
        assert_eq!(calculate_stat(10, &modifiers), 16);
    }

    #[test]
    fn test_calculate_stat_whole_number_percentage() {
        // Test that percentage values >= 1.0 are treated as whole percentages
        // A character has 14 might with a 20% buff (stored as 20.0)
        // This should calculate to 14 * (1 + 0.20) = 16.8 ≈ 17
        let modifiers = vec![StatModifier {
            stat: "might".to_string(),
            value: 20.0,
            modifier_type: ModifierType::Percentage,
        }];
        assert_eq!(calculate_stat(14, &modifiers), 17);
    }

    #[test]
    fn test_calculate_stat_mixed_percentage_formats() {
        // Test mixing decimal (0.10) and whole number (20) percentage formats
        // Base: 20, Flat: +10, Percentage: 10% + 20% = 30%
        // Result: (20 + 10) * (1 + 0.30) = 39
        let modifiers = vec![
            StatModifier {
                stat: "resistance".to_string(),
                value: 10.0,
                modifier_type: ModifierType::Flat,
            },
            StatModifier {
                stat: "resistance".to_string(),
                value: 0.10, // Decimal format
                modifier_type: ModifierType::Percentage,
            },
            StatModifier {
                stat: "resistance".to_string(),
                value: 20.0, // Whole number format
                modifier_type: ModifierType::Percentage,
            },
        ];
        assert_eq!(calculate_stat(20, &modifiers), 39);
    }

    #[test]
    fn test_calculate_stat_negative_whole_number_percentage() {
        // Test negative percentage as whole number
        // Base: 10, Flat: +10, Percentage: -20%
        // Result: (10 + 10) * (1 - 0.20) = 16
        let modifiers = vec![
            StatModifier {
                stat: "defense".to_string(),
                value: 10.0,
                modifier_type: ModifierType::Flat,
            },
            StatModifier {
                stat: "defense".to_string(),
                value: -20.0, // -20% stored as whole number
                modifier_type: ModifierType::Percentage,
            },
        ];
        assert_eq!(calculate_stat(10, &modifiers), 16);
    }
}
