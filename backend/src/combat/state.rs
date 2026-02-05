use crate::error::AppError;
use crate::models::CreatureInCombat;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use super::helpers::{
    GS_ABILITY_COOLDOWNS, GS_ENEMY, GS_PLAYER_HEALTH, GS_PRIMARY_WEAPON_ENCHANTED,
    GS_SECONDARY_WEAPON_ENCHANTED, GS_STATUS, GS_TURN_NUMBER,
};

/// The outcome of a combat state change
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CombatOutcome {
    /// Combat continues - neither side has won
    Ongoing,
    /// Player has won - enemy health reached 0
    Victory,
    /// Player has lost - player health reached 0
    Defeat,
}

/// Combat status as stored in game state
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum CombatStatus {
    Started,
    Ongoing,
    Victory,
    Defeat,
}

impl CombatStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            CombatStatus::Started => "started",
            CombatStatus::Ongoing => "ongoing",
            CombatStatus::Victory => "victory",
            CombatStatus::Defeat => "defeat",
        }
    }
}

/// Encapsulated combat state that enforces health modifications through methods
/// that automatically check for victory/defeat conditions.
#[derive(Debug, Clone)]
pub struct CombatState {
    // Original JSON - preserved so we can merge our changes back
    original_json: serde_json::Value,

    // Player state
    player_health: i64,
    player_max_health: i64,

    // Enemy state
    enemy: CreatureInCombat,

    // Combat metadata
    turn_number: i64,
    status: CombatStatus,

    // Ability state
    ability_cooldowns: HashMap<String, i32>,

    // Weapon enchantments
    primary_weapon_enchanted: Option<String>,
    secondary_weapon_enchanted: Option<String>,
}

impl CombatState {
    /// Create a new CombatState from an enemy and player stats
    pub fn new(enemy: CreatureInCombat, player_health: i64, player_max_health: i64) -> Self {
        Self {
            original_json: serde_json::Value::Null,
            player_health,
            player_max_health,
            enemy,
            turn_number: 0,
            status: CombatStatus::Started,
            ability_cooldowns: HashMap::new(),
            primary_weapon_enchanted: None,
            secondary_weapon_enchanted: None,
        }
    }

    /// Parse CombatState from a JSON game state value
    pub fn from_json(game_state: &serde_json::Value, player_max_health: i64) -> Result<Self, AppError> {
        // Parse enemy
        let enemy: CreatureInCombat = serde_json::from_value(
            game_state
                .get(GS_ENEMY)
                .ok_or_else(|| AppError::validation_error("No enemy in game state"))?
                .clone(),
        )
        .map_err(|e| {
            AppError::validation_error(&format!("Failed to parse enemy from game state: {}", e))
        })?;

        // Parse player health (default to max if not set)
        let player_health = game_state
            .get(GS_PLAYER_HEALTH)
            .and_then(|v| v.as_i64())
            .unwrap_or(player_max_health);

        // Parse turn number
        let turn_number = game_state
            .get(GS_TURN_NUMBER)
            .and_then(|v| v.as_i64())
            .unwrap_or(0);

        // Parse status
        let status = game_state
            .get(GS_STATUS)
            .and_then(|v| v.as_str())
            .map(|s| match s {
                "started" => CombatStatus::Started,
                "ongoing" => CombatStatus::Ongoing,
                "victory" => CombatStatus::Victory,
                "defeat" => CombatStatus::Defeat,
                _ => CombatStatus::Started,
            })
            .unwrap_or(CombatStatus::Started);

        // Parse ability cooldowns
        let ability_cooldowns: HashMap<String, i32> = game_state
            .get(GS_ABILITY_COOLDOWNS)
            .and_then(|v| serde_json::from_value(v.clone()).ok())
            .unwrap_or_default();

        // Parse weapon enchantments
        let primary_weapon_enchanted = game_state
            .get(GS_PRIMARY_WEAPON_ENCHANTED)
            .and_then(|v| v.as_str())
            .filter(|s| !s.is_empty())
            .map(String::from);

        let secondary_weapon_enchanted = game_state
            .get(GS_SECONDARY_WEAPON_ENCHANTED)
            .and_then(|v| v.as_str())
            .filter(|s| !s.is_empty())
            .map(String::from);

        Ok(Self {
            original_json: game_state.clone(),
            player_health,
            player_max_health,
            enemy,
            turn_number,
            status,
            ability_cooldowns,
            primary_weapon_enchanted,
            secondary_weapon_enchanted,
        })
    }

    /// Convert the combat state back to a JSON value for persistence
    /// This merges our tracked changes back into the original JSON to preserve
    /// any fields we don't explicitly track (like combatLog, adventureId, etc.)
    pub fn to_json(&self) -> serde_json::Value {
        // Start with the original JSON to preserve all fields
        let mut state = self.original_json.clone();

        // Update the fields we manage
        state[GS_ENEMY] = serde_json::to_value(&self.enemy).unwrap_or_default();
        state[GS_PLAYER_HEALTH] = serde_json::json!(self.player_health);
        state[GS_TURN_NUMBER] = serde_json::json!(self.turn_number);
        state[GS_STATUS] = serde_json::json!(self.status.as_str());
        state[GS_ABILITY_COOLDOWNS] = serde_json::json!(self.ability_cooldowns);

        // Update weapon enchantments
        if let Some(ref enchant) = self.primary_weapon_enchanted {
            state[GS_PRIMARY_WEAPON_ENCHANTED] = serde_json::json!(enchant);
        }
        if let Some(ref enchant) = self.secondary_weapon_enchanted {
            state[GS_SECONDARY_WEAPON_ENCHANTED] = serde_json::json!(enchant);
        }

        state
    }

    // ============== Health Observation Methods ==============
    // These methods modify health and automatically check for victory/defeat

    /// Apply damage to the enemy and check for victory
    /// Returns the combat outcome after applying the damage
    pub fn damage_enemy(&mut self, amount: i64) -> CombatOutcome {
        self.enemy.health = (self.enemy.health - amount).max(0);
        self.check_outcome()
    }

    /// Apply damage to the player and check for defeat
    /// Returns the combat outcome after applying the damage
    pub fn damage_player(&mut self, amount: i64) -> CombatOutcome {
        self.player_health = (self.player_health - amount).max(0);
        self.check_outcome()
    }

    /// Heal the player (capped at max health)
    /// Returns the combat outcome (always Ongoing unless already decided)
    pub fn heal_player(&mut self, amount: i64) -> CombatOutcome {
        self.player_health = (self.player_health + amount).min(self.player_max_health);
        self.check_outcome()
    }

    /// Restore player to full health
    pub fn restore_player_health(&mut self) {
        self.player_health = self.player_max_health;
    }

    /// Check the current combat outcome based on health values
    fn check_outcome(&self) -> CombatOutcome {
        if self.enemy.health <= 0 {
            CombatOutcome::Victory
        } else if self.player_health <= 0 {
            CombatOutcome::Defeat
        } else {
            CombatOutcome::Ongoing
        }
    }

    /// Get the current outcome without modifying state
    pub fn outcome(&self) -> CombatOutcome {
        self.check_outcome()
    }

    // ============== Status Management ==============

    /// Update the combat status based on outcome
    pub fn update_status(&mut self, outcome: CombatOutcome) {
        self.status = match outcome {
            CombatOutcome::Victory => CombatStatus::Victory,
            CombatOutcome::Defeat => CombatStatus::Defeat,
            CombatOutcome::Ongoing => {
                if self.turn_number <= 1 {
                    CombatStatus::Started
                } else {
                    CombatStatus::Ongoing
                }
            }
        };
    }

    /// Advance to the next turn
    pub fn advance_turn(&mut self) {
        self.turn_number += 1;
        // Update status to ongoing if we've progressed past the first turn
        if self.turn_number > 1 && self.status == CombatStatus::Started {
            self.status = CombatStatus::Ongoing;
        }
    }

    // ============== Accessors ==============

    pub fn player_health(&self) -> i64 {
        self.player_health
    }

    pub fn player_max_health(&self) -> i64 {
        self.player_max_health
    }

    pub fn enemy(&self) -> &CreatureInCombat {
        &self.enemy
    }

    pub fn enemy_health(&self) -> i64 {
        self.enemy.health
    }

    pub fn enemy_defense(&self) -> i64 {
        self.enemy.defense
    }

    pub fn enemy_might(&self) -> i64 {
        self.enemy.might
    }

    pub fn enemy_name(&self) -> &str {
        &self.enemy.name
    }

    pub fn enemy_experience_reward(&self) -> i64 {
        self.enemy.experience_reward
    }

    pub fn enemy_attack_description(&self) -> &str {
        &self.enemy.attack_description
    }

    pub fn turn_number(&self) -> i64 {
        self.turn_number
    }

    pub fn status(&self) -> CombatStatus {
        self.status
    }

    pub fn ability_cooldowns(&self) -> &HashMap<String, i32> {
        &self.ability_cooldowns
    }

    pub fn ability_cooldowns_mut(&mut self) -> &mut HashMap<String, i32> {
        &mut self.ability_cooldowns
    }

    pub fn primary_weapon_enchanted(&self) -> Option<&str> {
        self.primary_weapon_enchanted.as_deref()
    }

    pub fn secondary_weapon_enchanted(&self) -> Option<&str> {
        self.secondary_weapon_enchanted.as_deref()
    }

    pub fn set_primary_weapon_enchanted(&mut self, enchant: Option<String>) {
        self.primary_weapon_enchanted = enchant;
    }

    pub fn set_secondary_weapon_enchanted(&mut self, enchant: Option<String>) {
        self.secondary_weapon_enchanted = enchant;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_enemy() -> CreatureInCombat {
        CreatureInCombat {
            name: "Test Goblin".to_string(),
            health: 50,
            max_health: 50,
            level: 1,
            introduction_text: "A goblin appears!".to_string(),
            might: 10,
            defense: 5,
            magic: 3,
            resistance: 3,
            agility: 8,
            creature_type: Some("humanoid".to_string()),
            experience_reward: 100,
            attack_description: "The goblin attacks!".to_string(),
        }
    }

    #[test]
    fn test_combat_state_new() {
        let enemy = create_test_enemy();
        let state = CombatState::new(enemy, 100, 100);

        assert_eq!(state.player_health(), 100);
        assert_eq!(state.player_max_health(), 100);
        assert_eq!(state.enemy_health(), 50);
        assert_eq!(state.turn_number(), 0);
        assert_eq!(state.status(), CombatStatus::Started);
    }

    #[test]
    fn test_damage_enemy_ongoing() {
        let enemy = create_test_enemy();
        let mut state = CombatState::new(enemy, 100, 100);

        let outcome = state.damage_enemy(20);

        assert_eq!(outcome, CombatOutcome::Ongoing);
        assert_eq!(state.enemy_health(), 30);
    }

    #[test]
    fn test_damage_enemy_victory() {
        let enemy = create_test_enemy();
        let mut state = CombatState::new(enemy, 100, 100);

        let outcome = state.damage_enemy(50);

        assert_eq!(outcome, CombatOutcome::Victory);
        assert_eq!(state.enemy_health(), 0);
    }

    #[test]
    fn test_damage_enemy_overkill_clamps_to_zero() {
        let enemy = create_test_enemy();
        let mut state = CombatState::new(enemy, 100, 100);

        let outcome = state.damage_enemy(100); // More than enemy has

        assert_eq!(outcome, CombatOutcome::Victory);
        assert_eq!(state.enemy_health(), 0); // Clamped to 0, not -50
    }

    #[test]
    fn test_damage_player_ongoing() {
        let enemy = create_test_enemy();
        let mut state = CombatState::new(enemy, 100, 100);

        let outcome = state.damage_player(30);

        assert_eq!(outcome, CombatOutcome::Ongoing);
        assert_eq!(state.player_health(), 70);
    }

    #[test]
    fn test_damage_player_defeat() {
        let enemy = create_test_enemy();
        let mut state = CombatState::new(enemy, 100, 100);

        let outcome = state.damage_player(100);

        assert_eq!(outcome, CombatOutcome::Defeat);
        assert_eq!(state.player_health(), 0);
    }

    #[test]
    fn test_damage_player_overkill_clamps_to_zero() {
        let enemy = create_test_enemy();
        let mut state = CombatState::new(enemy, 100, 100);

        let outcome = state.damage_player(150);

        assert_eq!(outcome, CombatOutcome::Defeat);
        assert_eq!(state.player_health(), 0);
    }

    #[test]
    fn test_heal_player() {
        let enemy = create_test_enemy();
        let mut state = CombatState::new(enemy, 50, 100);

        let outcome = state.heal_player(30);

        assert_eq!(outcome, CombatOutcome::Ongoing);
        assert_eq!(state.player_health(), 80);
    }

    #[test]
    fn test_heal_player_capped_at_max() {
        let enemy = create_test_enemy();
        let mut state = CombatState::new(enemy, 80, 100);

        state.heal_player(50); // Would be 130, but capped at 100

        assert_eq!(state.player_health(), 100);
    }

    #[test]
    fn test_restore_player_health() {
        let enemy = create_test_enemy();
        let mut state = CombatState::new(enemy, 30, 100);

        state.restore_player_health();

        assert_eq!(state.player_health(), 100);
    }

    #[test]
    fn test_advance_turn() {
        let enemy = create_test_enemy();
        let mut state = CombatState::new(enemy, 100, 100);

        assert_eq!(state.turn_number(), 0);
        assert_eq!(state.status(), CombatStatus::Started);

        state.advance_turn();
        assert_eq!(state.turn_number(), 1);
        assert_eq!(state.status(), CombatStatus::Started);

        state.advance_turn();
        assert_eq!(state.turn_number(), 2);
        assert_eq!(state.status(), CombatStatus::Ongoing);
    }

    #[test]
    fn test_update_status_victory() {
        let enemy = create_test_enemy();
        let mut state = CombatState::new(enemy, 100, 100);

        state.update_status(CombatOutcome::Victory);

        assert_eq!(state.status(), CombatStatus::Victory);
    }

    #[test]
    fn test_update_status_defeat() {
        let enemy = create_test_enemy();
        let mut state = CombatState::new(enemy, 100, 100);

        state.update_status(CombatOutcome::Defeat);

        assert_eq!(state.status(), CombatStatus::Defeat);
    }

    #[test]
    fn test_victory_takes_priority_over_defeat() {
        // Edge case: both hit 0 at same time (player attacks and kills enemy)
        // Victory should be checked first since enemy health is evaluated first
        let mut enemy = create_test_enemy();
        enemy.health = 10;
        let mut state = CombatState::new(enemy, 10, 100);

        // Player attacks and kills enemy
        let outcome = state.damage_enemy(10);

        assert_eq!(outcome, CombatOutcome::Victory);
    }

    #[test]
    fn test_from_json() {
        let game_state = serde_json::json!({
            "enemy": {
                "name": "Test Goblin",
                "health": 30,
                "maxHealth": 50,
                "level": 1,
                "introductionText": "A goblin appears!",
                "might": 10,
                "defense": 5,
                "magic": 3,
                "resistance": 3,
                "agility": 8,
                "creatureType": "humanoid",
                "experienceReward": 100,
                "attackDescription": "The goblin attacks!"
            },
            "playerHealth": 75,
            "turnNumber": 3,
            "status": "ongoing",
            "abilityCooldowns": {
                "fireball": 2
            },
            "primaryWeaponEnchanted": "fire"
        });

        let state = CombatState::from_json(&game_state, 100).unwrap();

        assert_eq!(state.player_health(), 75);
        assert_eq!(state.player_max_health(), 100);
        assert_eq!(state.enemy_health(), 30);
        assert_eq!(state.turn_number(), 3);
        assert_eq!(state.status(), CombatStatus::Ongoing);
        assert_eq!(state.ability_cooldowns().get("fireball"), Some(&2));
        assert_eq!(state.primary_weapon_enchanted(), Some("fire"));
        assert_eq!(state.secondary_weapon_enchanted(), None);
    }

    #[test]
    fn test_to_json() {
        let enemy = create_test_enemy();
        let mut state = CombatState::new(enemy, 100, 100);
        state.advance_turn();
        state.damage_enemy(20);
        state.set_primary_weapon_enchanted(Some("frost".to_string()));

        let json = state.to_json();

        assert_eq!(json["playerHealth"], 100);
        assert_eq!(json["turnNumber"], 1);
        assert_eq!(json["status"], "started");
        assert_eq!(json["primaryWeaponEnchanted"], "frost");
        assert_eq!(json["enemy"]["health"], 30);
    }

    #[test]
    fn test_weapon_enchantments() {
        let enemy = create_test_enemy();
        let mut state = CombatState::new(enemy, 100, 100);

        assert_eq!(state.primary_weapon_enchanted(), None);
        assert_eq!(state.secondary_weapon_enchanted(), None);

        state.set_primary_weapon_enchanted(Some("fire".to_string()));
        state.set_secondary_weapon_enchanted(Some("frost".to_string()));

        assert_eq!(state.primary_weapon_enchanted(), Some("fire"));
        assert_eq!(state.secondary_weapon_enchanted(), Some("frost"));

        state.set_primary_weapon_enchanted(None);
        assert_eq!(state.primary_weapon_enchanted(), None);
    }
}
