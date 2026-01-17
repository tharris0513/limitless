use serde::{Deserialize, Serialize};
/// Character stats structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CharacterStats {
    pub might: i64,      // Physical power
    pub defense: i64,    // Physical defense
    pub magic: i64,      // Magical power
    pub resistance: i64, // Magical defense
    pub agility: i64,    // Speed
    pub adventures: i64,
    pub health: i64,
    #[serde(rename = "maxHealth")]
    pub max_health: i64,
    pub mana: i64,
    #[serde(rename = "maxMana")]
    pub max_mana: i64,
}

/// Character represents a game character that belongs to a user
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Character {
    pub id: String, // Character UUID
    #[serde(rename = "userId")]
    pub user_id: String, // Foreign key to User
    pub name: String, // Character name (chosen by player)
    #[serde(rename = "classId")]
    pub class_id: String, // Character's class
    pub level: i64,
    pub experience: i64,
    #[serde(rename = "experienceToNext")]
    pub experience_to_next: i64,
    pub stats: CharacterStats,
    pub location: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "gameState")]
    pub game_state: Option<String>, // JSON string storing current game state
    #[serde(rename = "createdAt")]
    pub created_at: String,
    #[serde(rename = "lastPlayed")]
    pub last_played: String,
}

/// Class represents a character class with base stats
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Class {
    pub id: String,
    pub name: String,
    pub description: String,
    #[serde(rename = "startingMight")]
    pub starting_might: i64,
    #[serde(rename = "startingDefense")]
    pub starting_defense: i64,
    #[serde(rename = "startingMagic")]
    pub starting_magic: i64,
    #[serde(rename = "startingResistance")]
    pub starting_resistance: i64,
    #[serde(rename = "startingAgility")]
    pub starting_agility: i64,
    #[serde(rename = "startingHealth")]
    pub starting_health: i64,
    #[serde(rename = "startingMana")]
    pub starting_mana: i64,
}
