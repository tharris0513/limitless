use crate::{error::AppError, game::creatures::CreatureInCombat};
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
    /// Parse CombatState from a JSON game state value
    pub fn from_json(
        game_state: &serde_json::Value,
        player_max_health: i64,
    ) -> Result<Self, AppError> {
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

    pub fn ability_cooldowns(&self) -> &HashMap<String, i32> {
        &self.ability_cooldowns
    }

    pub fn ability_cooldowns_mut(&mut self) -> &mut HashMap<String, i32> {
        &mut self.ability_cooldowns
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
        assert_eq!(state.enemy_health(), 30);
        assert_eq!(state.ability_cooldowns().get("fireball"), Some(&2));
    }
}
