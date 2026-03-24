use serde::{Deserialize, Serialize};

pub mod a_misty_eyed_dweller;

/// Creature represents an enemy or NPC that can be encountered
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Creature {
    pub id: &'static str,
    pub name: &'static str,
    pub introduction_text: &'static str,
    pub level: i64,
    pub max_health: i64,
    pub might: i64,
    pub defense: i64,
    pub magic: i64,
    pub resistance: i64,
    pub agility: i64,
    pub experience_reward: i64,
    pub creature_type: &'static str, // 'beast', 'undead', 'humanoid', 'elemental', 'dragon', 'demon', 'horror'
    pub attack_description: &'static str, // Template for attack text, e.g., "The creature strikes for ${damage} damage!"
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
