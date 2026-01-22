use serde::{Deserialize, Serialize};

/// Creature represents an enemy or NPC that can be encountered
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Creature {
    pub id: String,
    pub name: String,
    pub introduction_text: String,
    pub level: i64,
    pub max_health: i64,
    pub might: i64,
    pub defense: i64,
    pub magic: i64,
    pub resistance: i64,
    pub agility: i64,
    pub experience_reward: i64,
    pub creature_type: String, // 'beast', 'undead', 'humanoid', 'elemental', 'dragon', 'demon', 'horror'
    pub attack_description: Option<String>, // Template for attack text, e.g., "The creature strikes for ${damage} damage!"
    pub created_at: String,
}

/// Creature represents an enemy in the middle of combat
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreatureInCombat {
    pub name: String,
    pub introduction_text: String,
    pub level: i64,
    // only matters during combat
    pub health: i64,
    pub max_health: i64,
    pub might: i64,
    pub defense: i64,
    pub magic: i64,
    pub resistance: i64,
    pub agility: i64,
    pub creature_type: Option<String>,
    pub experience_reward: i64,
    pub attack_description: String,
}

/// Request DTO for creating a creature (without id, health, and createdAt)
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateCreatureRequest {
    pub name: String,
    pub introduction_text: String,
    pub level: i64,
    pub max_health: i64,
    pub might: i64,
    pub defense: i64,
    pub magic: i64,
    pub resistance: i64,
    pub agility: i64,
    pub experience_reward: i64,
    pub creature_type: String,
    pub attack_description: Option<String>,
}
