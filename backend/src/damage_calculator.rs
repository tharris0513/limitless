use crate::models::{Ability, Character, CharacterStats};
use evalexpr::*;
use rand::Rng;

pub struct DamageCalculator;

impl DamageCalculator {
    /// Calculate damage from an ability
    pub fn calculate_damage(
        ability: &Ability,
        caster: &Character,
        target: Option<&Character>,
    ) -> Result<i32, String> {
        if let Some(formula) = &ability.damage_formula {
            let context = Self::build_context(&caster.stats, caster.level, target);
            Self::evaluate_formula(formula, &context)
        } else {
            Ok(0)
        }
    }

    /// Calculate healing from an ability
    pub fn calculate_heal(ability: &Ability, caster: &Character) -> Result<i32, String> {
        if let Some(formula) = &ability.heal_formula {
            let context = Self::build_context(&caster.stats, caster.level, None);
            Self::evaluate_formula(formula, &context)
        } else {
            Ok(0)
        }
    }

    /// Calculate effect value from an ability
    pub fn calculate_effect(
        ability: &Ability,
        caster: &Character,
        target: Option<&Character>,
    ) -> Result<i32, String> {
        if let Some(formula) = &ability.effect_formula {
            let context = Self::build_context(&caster.stats, caster.level, target);
            Self::evaluate_formula(formula, &context)
        } else {
            Ok(0)
        }
    }

    /// Validate a formula by testing it with sample stats
    pub fn validate_formula(formula: &str) -> Result<(), String> {
        // Create sample stats for validation
        let sample_stats = CharacterStats {
            might: 10,
            defense: 10,
            magic: 10,
            resistance: 10,
            agility: 10,
            adventures: 0,
        };

        // Try to evaluate the formula with sample data
        // This catches both parsing and evaluation errors
        Self::test_formula(formula, &sample_stats, 1)?;
        Ok(())
    }

    /// Test a formula with sample stats
    pub fn test_formula(
        formula: &str,
        sample_stats: &CharacterStats,
        level: i64,
    ) -> Result<i32, String> {
        let context = Self::build_context(sample_stats, level, None);
        Self::evaluate_formula(formula, &context)
    }

    // Private helper methods

    fn evaluate_formula(formula: &str, context: &HashMapContext) -> Result<i32, String> {
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

        Ok(final_result.max(0.0) as i32) // Ensure non-negative and convert to i32
    }

    fn build_context(
        caster_stats: &CharacterStats,
        caster_level: i64,
        target: Option<&Character>,
    ) -> HashMapContext {
        let mut context = HashMapContext::new();

        // Caster stats
        context
            .set_value("might".into(), Value::from(caster_stats.might))
            .unwrap();
        context
            .set_value("defense".into(), Value::from(caster_stats.defense))
            .unwrap();
        context
            .set_value("magic".into(), Value::from(caster_stats.magic))
            .unwrap();
        context
            .set_value("resistance".into(), Value::from(caster_stats.resistance))
            .unwrap();
        context
            .set_value("agility".into(), Value::from(caster_stats.agility))
            .unwrap();
        context
            .set_value("level".into(), Value::from(caster_level))
            .unwrap();

        // Target stats (if available)
        if let Some(target) = target {
            context
                .set_value("target_defense".into(), Value::from(target.stats.defense))
                .unwrap();
            context
                .set_value(
                    "target_resistance".into(),
                    Value::from(target.stats.resistance),
                )
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
        }

        context
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_stats() -> CharacterStats {
        CharacterStats {
            might: 12,
            defense: 8,
            magic: 10,
            resistance: 6,
            agility: 15,
            adventures: 0,
        }
    }

    #[test]
    fn test_simple_damage_formula() {
        let stats = create_test_stats();
        let result = DamageCalculator::test_formula("(might * 0.8) + 15", &stats, 1);
        // Result will vary due to automatic ±10% variation (21-26)
        assert!(result.is_ok());
        let damage = result.unwrap();
        assert!(damage >= 21 && damage <= 27);
    }

    #[test]
    fn test_magic_damage_formula() {
        let stats = create_test_stats();
        let result = DamageCalculator::test_formula("(magic * 1.2) + (level * 3)", &stats, 5);
        // Result will vary due to automatic ±10% variation (24-29)
        assert!(result.is_ok());
        let damage = result.unwrap();
        assert!(damage >= 24 && damage <= 30);
    }

    #[test]
    fn test_hybrid_formula() {
        let stats = create_test_stats();
        let result =
            DamageCalculator::test_formula("(might * 0.5) + (magic * 0.5) + 20", &stats, 1);
        // Result will vary due to automatic ±10% variation (27-34)
        assert!(result.is_ok());
        let damage = result.unwrap();
        assert!(damage >= 27 && damage <= 35);
    }

    #[test]
    fn test_validate_formula() {
        assert!(DamageCalculator::validate_formula("might * 2").is_ok());
        assert!(DamageCalculator::validate_formula("(might + magic) / 2").is_ok());
        // Now properly catches invalid syntax
        assert!(DamageCalculator::validate_formula("might +").is_err());
        assert!(DamageCalculator::validate_formula("might *").is_err());
        assert!(DamageCalculator::validate_formula("+ might").is_err());
    }

    #[test]
    fn test_negative_damage_prevented() {
        let stats = create_test_stats();
        let result = DamageCalculator::test_formula("might - 100", &stats, 1);
        assert_eq!(result.unwrap(), 0); // Negative damage becomes 0
    }

    #[test]
    fn test_rand_function() {
        let stats = create_test_stats();
        // Test rand() function - should return value between 0 and base damage
        let result = DamageCalculator::test_formula("10 * rand()", &stats, 1);
        assert!(result.is_ok());
        let damage = result.unwrap();
        assert!(damage >= 0 && damage <= 10);
    }

    #[test]
    fn test_rand_range_function() {
        let stats = create_test_stats();
        // Test randRange() - should return value between min and max
        let result = DamageCalculator::test_formula("might * randRange(1.5, 2.0)", &stats, 1);
        assert!(result.is_ok());
        let damage = result.unwrap();
        // might=12, so 12 * 1.5 = 18 to 12 * 2.0 = 24
        assert!(damage >= 18 && damage <= 24);
    }

    #[test]
    fn test_automatic_variation() {
        let stats = create_test_stats();
        // Formula without rand() should have automatic ±10% variation
        // Base formula: might * 2 = 12 * 2 = 24
        // With ±10% variation: 21.6 to 26.4
        let result = DamageCalculator::test_formula("might * 2", &stats, 1);
        assert!(result.is_ok());
        let damage = result.unwrap();
        assert!(damage >= 21 && damage <= 27);
    }
}
