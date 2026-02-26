/// Experience and leveling system for characters
///
/// This module provides utilities for calculating experience requirements
/// and managing character progression through levels.

/// Calculate the experience required to reach a specific level.
///
/// Uses a slow exponential curve: base_exp * ((level-1)^1.5)
/// This creates a progression where early levels are quick but later levels
/// take significantly more experience.
///
/// Examples:
/// - Level 1 -> 2: 100 XP
/// - Level 2 -> 3: 283 XP
/// - Level 5 -> 6: 1,118 XP
/// - Level 10 -> 11: 3,162 XP
/// - Level 20 -> 21: 8,944 XP
pub fn experience_for_level(level: i64) -> i64 {
    if level <= 1 {
        return 0;
    }

    let base_exp = 100.0;
    let exponent = 1.5;

    (base_exp * ((level - 1) as f64).powf(exponent)).round() as i64
}

/// Calculate what level a character should be based on their total experience.
/// Returns (current_level, experience_into_current_level, experience_needed_for_next_level)
pub fn calculate_level_from_experience(total_exp: i64) -> (i64, i64, i64) {
    let mut level = 1;
    let mut accumulated_exp = 0;

    // Keep leveling up until we run out of experience
    loop {
        let exp_for_next = experience_for_level(level + 1);
        if accumulated_exp + exp_for_next > total_exp {
            // Not enough experience for next level
            let exp_into_level = total_exp - accumulated_exp;
            return (level, exp_into_level, exp_for_next);
        }

        accumulated_exp += exp_for_next;
        level += 1;

        // Safety cap at level 100
        if level >= 100 {
            return (100, 0, i64::MAX);
        }
    }
}

/// Calculate stat increases for a character when they level up.
/// Returns (might, defense, magic, resistance, agility, max_health, max_mana)
///
/// Each level grants:
/// - Primary stats (based on class): +2
/// - Secondary stats: +1
/// - Health: +10 per level
/// - Mana: +5 per level
///
/// For now, all characters get balanced stat increases.
/// TODO: Make this class-specific in the future.
pub fn calculate_stat_increases_for_level(
    _class_id: &str,
    _new_level: i64,
) -> (i64, i64, i64, i64, i64, i64, i64) {
    // Balanced increases for all classes
    // Future enhancement: make this class-specific
    let might = 2;
    let defense = 1;
    let magic = 2;
    let resistance = 1;
    let agility = 1;
    let max_health = 10;
    let max_mana = 5;

    (
        might, defense, magic, resistance, agility, max_health, max_mana,
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_experience_curve() {
        // Level 1 -> 2 should be 100
        assert_eq!(experience_for_level(2), 100);

        // Level 2 -> 3 should be around 283
        assert_eq!(experience_for_level(3), 283);

        // Level 10 -> 11 should be around 3,162
        assert_eq!(experience_for_level(11), 3162);

        // Verify it's exponential (later levels cost more)
        assert!(experience_for_level(20) > experience_for_level(10) * 2);
    }

    #[test]
    fn test_calculate_level_from_experience() {
        // 0 XP = level 1
        let (level, exp_into, exp_needed) = calculate_level_from_experience(0);
        assert_eq!(level, 1);
        assert_eq!(exp_into, 0);
        assert_eq!(exp_needed, 100);

        // 50 XP = level 1 (halfway to level 2)
        let (level, exp_into, exp_needed) = calculate_level_from_experience(50);
        assert_eq!(level, 1);
        assert_eq!(exp_into, 50);
        assert_eq!(exp_needed, 100);

        // 100 XP = level 2
        let (level, exp_into, exp_needed) = calculate_level_from_experience(100);
        assert_eq!(level, 2);
        assert_eq!(exp_into, 0);
        assert_eq!(exp_needed, 283);

        // 150 XP = level 2 (partway to level 3)
        let (level, exp_into, exp_needed) = calculate_level_from_experience(150);
        assert_eq!(level, 2);
        assert_eq!(exp_into, 50);
        assert_eq!(exp_needed, 283);
    }
}
