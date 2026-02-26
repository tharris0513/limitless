use crate::game::abilities::ActiveBuff;
use serde::{Deserialize, Serialize};

/// Character represents a game character that belongs to a user
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Character {
    pub id: String,       // Character UUID
    pub user_id: String,  // Foreign key to User
    pub name: String,     // Character name (chosen by player)
    pub class_id: String, // Character's class
    pub level: i64,
    pub experience: i64,
    pub experience_to_next: i64,

    #[serde(skip_deserializing)]
    pub health: i64,
    pub max_health: i64,
    #[serde(skip_deserializing)]
    pub mana: i64,
    pub max_mana: i64,
    pub might: i64,      // Physical power
    pub defense: i64,    // Physical defense
    pub magic: i64,      // Magical power
    pub resistance: i64, // Magical defense
    pub agility: i64,    // Speed
    pub adventures: i64,

    pub location: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "gameState")]
    pub game_state: Option<String>, // JSON string storing current game state
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "activeBuffs")]
    pub active_buffs: Option<Vec<ActiveBuff>>, // Currently active noncombat ability buffs
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "calculatedStats")]
    pub calculated_stats: Option<crate::stat_calculator::CalculatedStats>, // Stats including all modifiers
    #[serde(rename = "createdAt")]
    pub created_at: String,
    #[serde(rename = "lastPlayed")]
    pub last_played: String,
}
