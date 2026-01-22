use crate::models::{Ability, Character, Creature, CreatureInCombat};
use evalexpr::*;
use rand::Rng;

pub struct DamageCalculator;

impl DamageCalculator {
    /// Calculate damage from an ability
    pub fn calculate_damage(
        ability: &Ability,
        caster: &Character,
        target: Option<&Character>,
    ) -> Result<i64, String> {
        if let Some(formula) = &ability.damage_formula {
            let context = Self::build_context(&caster, target);
            Self::evaluate_formula(formula, &context)
        } else {
            Ok(0)
        }
    }

    /// Calculate healing from an ability
    pub fn calculate_heal(ability: &Ability, caster: &Character) -> Result<i64, String> {
        if let Some(formula) = &ability.heal_formula {
            let context = Self::build_context(&caster, None);
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
    ) -> Result<i64, String> {
        if let Some(formula) = &ability.effect_formula {
            let context = Self::build_context(&caster, target);
            Self::evaluate_formula(formula, &context)
        } else {
            Ok(0)
        }
    }

    /// Calculate melee attack damage based on might with variance, reduced by target defense
    pub fn calculate_melee_attack(might: i64, target_defense: i64) -> i64 {
        use rand::Rng;
        let mut rng = rand::thread_rng();
        let variance = rng.gen_range(-0.1..=0.1);
        let raw_damage = ((might as f64) * (1.0 + variance)).round() as i64;
        // Subtract target defense (minimum 1 damage)
        (raw_damage - target_defense).max(1)
    }

    /// Validate a formula by testing it with sample stats
    pub fn validate_formula(formula: &str) -> Result<(), String> {
        // Create sample stats for validation
        let sample_character = Character {
            id: "sample_id".to_string(),
            user_id: "sample_user".to_string(),
            name: "Sample".to_string(),
            class_id: "sample_class".to_string(),
            level: 1,
            experience: 0,
            experience_to_next: 100,
            location: "sample_location".to_string(),
            game_state: Some("{}".to_string()),
            active_buffs: Some(vec![].into()),
            created_at: "".to_string(),
            last_played: "".to_string(),
            might: 10,
            defense: 10,
            magic: 10,
            resistance: 10,
            agility: 10,
            adventures: 0,
            health: 100,
            max_health: 100,
            mana: 50,
            max_mana: 50,
        };

        // Try to evaluate the formula with sample data
        // This catches both parsing and evaluation errors
        Self::test_formula(formula, &sample_character)?;
        Ok(())
    }

    /// Test a formula with sample stats
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

        // Caster stats
        context
            .set_value("might".into(), Value::from(caster.might))
            .unwrap();
        context
            .set_value("defense".into(), Value::from(caster.defense))
            .unwrap();
        context
            .set_value("magic".into(), Value::from(caster.magic))
            .unwrap();
        context
            .set_value("resistance".into(), Value::from(caster.resistance))
            .unwrap();
        context
            .set_value("agility".into(), Value::from(caster.agility))
            .unwrap();
        context
            .set_value("level".into(), Value::from(caster.level))
            .unwrap();

        // Target stats (if available)
        if let Some(target) = target {
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
        }

        context
    }

    fn build_context_with_enemy(caster: &Character, target: &CreatureInCombat) -> HashMapContext {
        let mut context = HashMapContext::new();

        // Caster stats
        context
            .set_value("might".into(), Value::from(caster.might))
            .unwrap();
        context
            .set_value("defense".into(), Value::from(caster.defense))
            .unwrap();
        context
            .set_value("magic".into(), Value::from(caster.magic))
            .unwrap();
        context
            .set_value("resistance".into(), Value::from(caster.resistance))
            .unwrap();
        context
            .set_value("agility".into(), Value::from(caster.agility))
            .unwrap();
        context
            .set_value("level".into(), Value::from(caster.level))
            .unwrap();

        // Enemy stats
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
    use crate::models::CreatureInCombat;

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
            creature_type: "beast".to_string(),
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
            creature_type: "beast".to_string(),
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
}
