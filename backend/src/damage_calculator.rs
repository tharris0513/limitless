use crate::{game::creatures::CreatureInCombat, models::Character};
use evalexpr::*;
use rand::Rng;

pub struct DamageCalculator;

impl DamageCalculator {
    /// Calculate melee attack damage based on might with variance, reduced by target defense
    pub fn calculate_melee_attack(might: i64, target_defense: i64) -> i64 {
        use rand::Rng;
        let mut rng = rand::thread_rng();
        let variance = rng.gen_range(-0.1..=0.1);
        let raw_damage = ((might as f64) * (1.0 + variance)).round() as i64;
        // Subtract target defense (minimum 1 damage)
        (raw_damage - target_defense).max(1)
    }

    /// Test a formula with sample stats
    /// Used for testing formulas in the test suite
    #[allow(dead_code)]
    pub fn test_formula(formula: &str, sample_character: &Character) -> Result<i64, String> {
        let context = Self::build_context(sample_character, None);
        Self::evaluate_formula(formula, &context)
    }

    /// Test a formula with sample stats and enemy stats
    pub fn test_formula_with_enemy(
        formula: &str,
        caster: &Character,
        target: &CreatureInCombat,
    ) -> Result<i64, String> {
        let context = Self::build_context_with_enemy(caster, target);
        Self::evaluate_formula(formula, &context)
    }

    // Private helper methods

    fn evaluate_formula(formula: &str, context: &HashMapContext) -> Result<i64, String> {
        let mut context_with_functions = context.clone();

        // Add rand() function - returns random float between 0.0 and 1.0
        context_with_functions
            .set_function(
                "rand".into(),
                Function::new(|argument| {
                    if !argument.is_empty() {
                        return Err(EvalexprError::WrongFunctionArgumentAmount {
                            expected: 0..=0,
                            actual: 1,
                        });
                    }
                    let mut rng = rand::thread_rng();
                    Ok(Value::from(rng.gen::<f64>()))
                }),
            )
            .unwrap();

        // Add randRange(min, max) function - returns random float between min and max
        context_with_functions
            .set_function(
                "randRange".into(),
                Function::new(|argument| {
                    let tuple = argument.as_tuple()?;
                    if tuple.len() != 2 {
                        return Err(EvalexprError::WrongFunctionArgumentAmount {
                            expected: 2..=2,
                            actual: tuple.len(),
                        });
                    }
                    let min = tuple[0].as_number()?;
                    let max = tuple[1].as_number()?;
                    let mut rng = rand::thread_rng();
                    Ok(Value::from(rng.gen_range(min..=max)))
                }),
            )
            .unwrap();

        // Add min(a, b) function - returns the minimum of two values
        context_with_functions
            .set_function(
                "min".into(),
                Function::new(|argument| {
                    let tuple = argument.as_tuple()?;
                    if tuple.len() != 2 {
                        return Err(EvalexprError::WrongFunctionArgumentAmount {
                            expected: 2..=2,
                            actual: tuple.len(),
                        });
                    }
                    let a = tuple[0].as_number()?;
                    let b = tuple[1].as_number()?;
                    Ok(Value::from(a.min(b)))
                }),
            )
            .unwrap();

        // Add max(a, b) function - returns the maximum of two values
        context_with_functions
            .set_function(
                "max".into(),
                Function::new(|argument| {
                    let tuple = argument.as_tuple()?;
                    if tuple.len() != 2 {
                        return Err(EvalexprError::WrongFunctionArgumentAmount {
                            expected: 2..=2,
                            actual: tuple.len(),
                        });
                    }
                    let a = tuple[0].as_number()?;
                    let b = tuple[1].as_number()?;
                    Ok(Value::from(a.max(b)))
                }),
            )
            .unwrap();

        let base_result = eval_with_context(formula, &context_with_functions)
            .map_err(|e| format!("Formula evaluation error: {}", e))?
            .as_number()
            .map_err(|_| "Formula must return a number".to_string())?;

        // Apply automatic ±10% damage variation if formula doesn't already include random
        let final_result = if !formula.contains("rand") {
            let mut rng = rand::thread_rng();
            let variation = rng.gen_range(0.9..=1.1);
            base_result * variation
        } else {
            base_result
        };

        Ok(final_result.max(0.0) as i64) // Ensure non-negative and convert to i64
    }

    fn build_context(caster: &Character, target: Option<&Character>) -> HashMapContext {
        let mut context = HashMapContext::new();

        // Use calculated stats if available, otherwise fall back to base stats
        let caster_stats = caster.calculated_stats.as_ref();

        // Caster stats
        context
            .set_value(
                "might".into(),
                Value::from(caster_stats.map(|s| s.might).unwrap_or(caster.might)),
            )
            .unwrap();
        context
            .set_value(
                "defense".into(),
                Value::from(caster_stats.map(|s| s.defense).unwrap_or(caster.defense)),
            )
            .unwrap();
        context
            .set_value(
                "magic".into(),
                Value::from(caster_stats.map(|s| s.magic).unwrap_or(caster.magic)),
            )
            .unwrap();
        context
            .set_value(
                "resistance".into(),
                Value::from(
                    caster_stats
                        .map(|s| s.resistance)
                        .unwrap_or(caster.resistance),
                ),
            )
            .unwrap();
        context
            .set_value(
                "agility".into(),
                Value::from(caster_stats.map(|s| s.agility).unwrap_or(caster.agility)),
            )
            .unwrap();
        context
            .set_value("level".into(), Value::from(caster.level))
            .unwrap();

        // Target stats (if available)
        if let Some(target) = target {
            let target_stats = target.calculated_stats.as_ref();
            context
                .set_value(
                    "target_defense".into(),
                    Value::from(target_stats.map(|s| s.defense).unwrap_or(target.defense)),
                )
                .unwrap();
            context
                .set_value(
                    "target_resistance".into(),
                    Value::from(
                        target_stats
                            .map(|s| s.resistance)
                            .unwrap_or(target.resistance),
                    ),
                )
                .unwrap();
            context
                .set_value("target_level".into(), Value::from(target.level))
                .unwrap();
            context
                .set_value("target_health".into(), Value::from(target.health))
                .unwrap();
            context
                .set_value(
                    "target_max_health".into(),
                    Value::from(
                        target_stats
                            .map(|s| s.max_health)
                            .unwrap_or(target.max_health),
                    ),
                )
                .unwrap();
        }

        context
    }

    fn build_context_with_enemy(caster: &Character, target: &CreatureInCombat) -> HashMapContext {
        let mut context = HashMapContext::new();

        // Use calculated stats if available, otherwise fall back to base stats
        let caster_stats = caster.calculated_stats.as_ref();

        // Caster stats
        context
            .set_value(
                "might".into(),
                Value::from(caster_stats.map(|s| s.might).unwrap_or(caster.might)),
            )
            .unwrap();
        context
            .set_value(
                "defense".into(),
                Value::from(caster_stats.map(|s| s.defense).unwrap_or(caster.defense)),
            )
            .unwrap();
        context
            .set_value(
                "magic".into(),
                Value::from(caster_stats.map(|s| s.magic).unwrap_or(caster.magic)),
            )
            .unwrap();
        context
            .set_value(
                "resistance".into(),
                Value::from(
                    caster_stats
                        .map(|s| s.resistance)
                        .unwrap_or(caster.resistance),
                ),
            )
            .unwrap();
        context
            .set_value(
                "agility".into(),
                Value::from(caster_stats.map(|s| s.agility).unwrap_or(caster.agility)),
            )
            .unwrap();
        context
            .set_value("level".into(), Value::from(caster.level))
            .unwrap();

        // Enemy stats (enemies don't have calculated stats, use raw stats)
        context
            .set_value("target_defense".into(), Value::from(target.defense))
            .unwrap();
        context
            .set_value("target_resistance".into(), Value::from(target.resistance))
            .unwrap();
        context
            .set_value("target_level".into(), Value::from(target.level))
            .unwrap();
        context
            .set_value("target_health".into(), Value::from(target.health))
            .unwrap();
        context
            .set_value("target_max_health".into(), Value::from(target.max_health))
            .unwrap();

        context
    }
}

#[cfg(test)]
mod tests {

    use super::*;

    fn create_test_character() -> Character {
        Character {
            id: "test_id".to_string(),
            user_id: "test_user".to_string(),
            name: "Test".to_string(),
            class_id: "test_class".to_string(),
            level: 5,
            experience: 0,
            experience_to_next: 100,
            location: "test_location".to_string(),
            game_state: Some("{}".to_string()),
            active_buffs: Some(vec![].into()),
            created_at: "".to_string(),
            last_played: "".to_string(),
            might: 12,
            defense: 8,
            magic: 10,
            resistance: 6,
            agility: 15,
            adventures: 0,
            health: 80,
            max_health: 100,
            mana: 30,
            max_mana: 50,
            calculated_stats: None,
        }
    }

    fn create_test_creature() -> CreatureInCombat {
        CreatureInCombat {
            name: "Test Creature".to_string(),
            introduction_text: "A fearsome creature.".to_string(),
            level: 3,
            health: 50,
            max_health: 50,
            might: 10,
            defense: 5,
            magic: 8,
            resistance: 4,
            agility: 12,
            experience_reward: 20,
            creature_type: Some("beast".to_string()),
            attack_description: "The creature lunges forward!".to_string(),
        }
    }

    fn create_test_creature_with_high_defense() -> CreatureInCombat {
        CreatureInCombat {
            name: "Tank Creature".to_string(),
            introduction_text: "A heavily armored creature.".to_string(),
            level: 3,
            health: 100,
            max_health: 100,
            might: 10,
            defense: 999999,
            magic: 8,
            resistance: 4,
            agility: 12,
            experience_reward: 20,
            creature_type: Some("beast".to_string()),
            attack_description: "The creature stands firm!".to_string(),
        }
    }

    #[test]
    fn test_simple_damage_formula() {
        let character = create_test_character();
        let result = DamageCalculator::test_formula("(might * 0.8) + 15", &character);
        // Result will vary due to automatic ±10% variation (21-26)
        assert!(result.is_ok());
        let damage = result.unwrap();
        assert!(damage >= 21 && damage <= 27);
    }

    #[test]
    fn test_magic_damage_formula() {
        let character = create_test_character();
        let result = DamageCalculator::test_formula("(magic * 1.2) + (level * 3)", &character);
        // Result will vary due to automatic ±10% variation (24-29)
        assert!(result.is_ok());
        let damage = result.unwrap();
        assert!(damage >= 24 && damage <= 30);
    }

    #[test]
    fn test_hybrid_formula() {
        let character = create_test_character();
        let result =
            DamageCalculator::test_formula("(might * 0.5) + (magic * 0.5) + 20", &character);
        // Result will vary due to automatic ±10% variation (27-34)
        assert!(result.is_ok());
        let damage = result.unwrap();
        assert!(damage >= 27 && damage <= 35);
    }

    #[test]
    fn test_negative_damage_prevented() {
        let character = create_test_character();
        let result = DamageCalculator::test_formula("might - 100", &character);
        assert_eq!(result.unwrap(), 0); // Negative damage becomes 0
    }

    #[test]
    fn test_rand_function() {
        let character = create_test_character();
        // Test rand() function - should return value between 0 and base damage
        let result = DamageCalculator::test_formula("10 * rand()", &character);
        assert!(result.is_ok());
        let damage = result.unwrap();
        assert!(damage >= 0 && damage <= 10);
    }

    #[test]
    fn test_rand_range_function() {
        let character = create_test_character();
        // Test randRange() - should return value between min and max
        let result = DamageCalculator::test_formula("might * randRange(1.5, 2.0)", &character);
        assert!(result.is_ok());
        let damage = result.unwrap();
        // might=12, so 12 * 1.5 = 18 to 12 * 2.0 = 24
        assert!(damage >= 18 && damage <= 24);
    }

    #[test]
    fn test_automatic_variation() {
        let character = create_test_character();
        // Formula without rand() should have automatic ±10% variation
        // Base formula: might * 2 = 12 * 2 = 24
        // With ±10% variation: 21.6 to 26.4
        let result = DamageCalculator::test_formula("might * 2", &character);
        assert!(result.is_ok());
        let damage = result.unwrap();
        assert!(damage >= 21 && damage <= 27);
    }

    #[test]
    fn test_min_function() {
        let character = create_test_character();
        // Test min() function with constants (use rand() to bypass auto-variation)
        let result = DamageCalculator::test_formula("min(10, 20) + rand()", &character);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 10);

        // Test min() function with stats (use rand() to bypass auto-variation)
        let result = DamageCalculator::test_formula("min(might, magic) + rand()", &character);
        assert!(result.is_ok());
        // might=12, magic=10, so min should be 10
        assert_eq!(result.unwrap(), 10);
    }

    #[test]
    fn test_max_function() {
        let character = create_test_character();
        // Test max() function with constants (use rand() to bypass auto-variation)
        let result = DamageCalculator::test_formula("max(10, 20) + rand()", &character);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 20);

        // Test max() function with stats (use rand() to bypass auto-variation)
        let result = DamageCalculator::test_formula("max(might, magic) + rand()", &character);
        assert!(result.is_ok());
        // might=12, magic=10, so max should be 12
        assert_eq!(result.unwrap(), 12);
    }

    #[test]
    fn test_max_prevents_negative_damage() {
        let character = create_test_character();
        // Test max() to prevent negative damage when defense is high
        // might=12, if we subtract a high value, max(0, ...) ensures non-negative
        let result = DamageCalculator::test_formula("max(0, might - 100) + rand()", &character);
        assert!(result.is_ok());
        let damage = result.unwrap();
        // Should be 0 since might - 100 = -88, max(0, -88) = 0
        assert_eq!(damage, 0);
    }

    #[test]
    fn test_min_caps_damage() {
        let character = create_test_character();
        // Test min() to cap damage at a maximum value
        let result = DamageCalculator::test_formula("min(50, might * 10) + rand()", &character);
        assert!(result.is_ok());
        let damage = result.unwrap();
        // might * 10 = 120, but min(50, 120) = 50
        assert_eq!(damage, 50);
    }

    #[test]
    fn test_min_max_combined() {
        let character = create_test_character();
        // Test combining min and max to clamp a value between bounds
        let result = DamageCalculator::test_formula("max(10, min(20, might)) + rand()", &character);
        assert!(result.is_ok());
        let damage = result.unwrap();
        // might=12, min(20, 12)=12, max(10, 12)=12
        assert_eq!(damage, 12);

        // Test with a value below the minimum
        let result = DamageCalculator::test_formula("max(15, min(20, magic)) + rand()", &character);
        assert!(result.is_ok());
        let damage = result.unwrap();
        // magic=10, min(20, 10)=10, max(15, 10)=15
        assert_eq!(damage, 15);
    }

    #[test]
    fn test_max_with_defense_subtraction() {
        let character = create_test_character();
        let target = create_test_creature();
        // Test realistic scenario: damage with defense mitigation
        let result = DamageCalculator::test_formula_with_enemy(
            "max(1, might - target_defense) + rand()",
            &character,
            &target,
        );
        assert!(result.is_ok());
        let damage = result.unwrap();
        // might=12, target_defense=5, so 12-5=7, max(1, 7)=7
        assert_eq!(damage, 7);

        // Test with very high defense
        let high_defense_target = create_test_creature_with_high_defense();
        let result = DamageCalculator::test_formula_with_enemy(
            "max(1, might - target_defense) + rand()",
            &character,
            &high_defense_target,
        );
        assert!(result.is_ok());
        let damage = result.unwrap();
        // might=12, target_defense=99999, so 12-99999=-99987, max(1, -99987)=1
        assert_eq!(damage, 1);
    }

    // Tests for calculated stats integration

    #[test]
    fn test_calculated_stats_might_buff() {
        let mut character = create_test_character();
        // Base might is 12, add calculated stats with might = 20
        character.calculated_stats = Some(crate::stat_calculator::CalculatedStats {
            might: 20,
            defense: 8,
            magic: 10,
            resistance: 6,
            agility: 15,
            max_health: 100,
            max_mana: 50,
        });

        // Test that formulas use calculated might (20) instead of base might (12)
        let result = DamageCalculator::test_formula("might * 2 + rand()", &character);
        assert!(result.is_ok());
        let damage = result.unwrap();
        // Calculated might is 20, so 20 * 2 = 40
        assert_eq!(damage, 40);
    }

    #[test]
    fn test_calculated_stats_magic_buff() {
        let mut character = create_test_character();
        // Base magic is 10, add calculated stats with magic = 25
        character.calculated_stats = Some(crate::stat_calculator::CalculatedStats {
            might: 12,
            defense: 8,
            magic: 25,
            resistance: 6,
            agility: 15,
            max_health: 100,
            max_mana: 50,
        });

        // Test that formulas use calculated magic (25) instead of base magic (10)
        let result = DamageCalculator::test_formula("(magic * 1.5) + rand()", &character);
        assert!(result.is_ok());
        let damage = result.unwrap();

        // Calculated magic is 25, so 25 * 1.5 = 37.5 ≈ 38
        // Result will vary due to automatic ±10% variation (34.2 to 41.4), but should be around 38
        assert!(damage >= 34 && damage <= 42);
    }

    #[test]
    fn test_calculated_stats_all_stats_buffed() {
        let mut character = create_test_character();
        // Apply buffs to all stats
        character.calculated_stats = Some(crate::stat_calculator::CalculatedStats {
            might: 20,       // was 12
            defense: 15,     // was 8
            magic: 18,       // was 10
            resistance: 12,  // was 6
            agility: 25,     // was 15
            max_health: 150, // was 100
            max_mana: 80,    // was 50
        });

        // Test hybrid formula using multiple stats
        let result = DamageCalculator::test_formula(
            "(might * 0.5) + (magic * 0.5) + agility + rand()",
            &character,
        );
        assert!(result.is_ok());
        let damage = result.unwrap();
        // (20 * 0.5) + (18 * 0.5) + 25 = 10 + 9 + 25 = 44
        assert_eq!(damage, 44);
    }

    #[test]
    fn test_calculated_stats_vs_base_stats() {
        let mut character_with_calc = create_test_character();
        let character_without_calc = create_test_character();

        // Add calculated stats with significantly higher might
        character_with_calc.calculated_stats = Some(crate::stat_calculator::CalculatedStats {
            might: 30,
            defense: 8,
            magic: 10,
            resistance: 6,
            agility: 15,
            max_health: 100,
            max_mana: 50,
        });

        // Test the same formula with both characters
        let result_with_calc =
            DamageCalculator::test_formula("might * 2 + rand()", &character_with_calc);
        let result_without_calc =
            DamageCalculator::test_formula("might * 2 + rand()", &character_without_calc);

        assert!(result_with_calc.is_ok());
        assert!(result_without_calc.is_ok());

        let damage_with_calc = result_with_calc.unwrap();
        let damage_without_calc = result_without_calc.unwrap();

        // Character with calculated stats should do more damage (30*2=60 vs 12*2=24)
        assert_eq!(damage_with_calc, 60);
        assert_eq!(damage_without_calc, 24);
    }

    #[test]
    fn test_melee_attack_with_calculated_stats() {
        let calculated_might = 25;
        let target_defense = 10;

        // Run melee attack calculation multiple times to account for variance
        let mut all_in_range = true;
        for _ in 0..10 {
            let damage = DamageCalculator::calculate_melee_attack(calculated_might, target_defense);
            // Raw damage: 25 * (0.9 to 1.1) = 22.5 to 27.5
            // After defense: (22.5 to 27.5) - 10 = 12.5 to 17.5
            // Minimum 1 damage
            if damage < 12 || damage > 18 {
                all_in_range = false;
                break;
            }
        }
        assert!(all_in_range);
    }

    #[test]
    fn test_calculated_stats_level_scaling() {
        let mut character = create_test_character(); // level 5

        // Buff stats significantly
        character.calculated_stats = Some(crate::stat_calculator::CalculatedStats {
            might: 25,
            defense: 15,
            magic: 20,
            resistance: 12,
            agility: 20,
            max_health: 150,
            max_mana: 80,
        });

        // Test formula that combines stats with level
        let result =
            DamageCalculator::test_formula("(might + magic) + (level * 2) + rand()", &character);
        assert!(result.is_ok());
        let damage = result.unwrap();
        // (25 + 20) + (5 * 2) = 45 + 10 = 55
        assert_eq!(damage, 55);
    }

    #[test]
    fn test_calculated_stats_complex_formula() {
        let mut character = create_test_character();

        character.calculated_stats = Some(crate::stat_calculator::CalculatedStats {
            might: 20,
            defense: 15,
            magic: 25,
            resistance: 10,
            agility: 30,
            max_health: 150,
            max_mana: 80,
        });

        // Test a complex formula using multiple calculated stats
        let result = DamageCalculator::test_formula(
            "max(might, magic) + (agility * 0.5) + min(defense, resistance) + rand()",
            &character,
        );
        assert!(result.is_ok());
        let damage = result.unwrap();
        // max(20, 25) + (30 * 0.5) + min(15, 10) = 25 + 15 + 10 = 50
        assert_eq!(damage, 50);
    }
}
