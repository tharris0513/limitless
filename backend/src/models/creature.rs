use serde::{Deserialize, Serialize};

/// Creature represents an enemy or NPC that can be encountered
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Creature {
    pub id: String,
    pub name: String,
    #[serde(rename = "introductionText")]
    pub introduction_text: String,
    pub level: i64,
    pub health: i64,
    pub might: i64,
    pub defense: i64,
    pub magic: i64,
    pub resistance: i64,
    pub agility: i64,
    #[serde(rename = "experienceReward")]
    pub experience_reward: i64,
    #[serde(rename = "creatureType")]
    pub creature_type: String, // 'beast', 'undead', 'humanoid', 'elemental', 'dragon', 'demon', 'horror'
    #[serde(rename = "createdAt")]
    pub created_at: String,
}

/// Request DTO for creating a creature (without id and createdAt)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateCreatureRequest {
    pub name: String,
    #[serde(rename = "introductionText")]
    pub introduction_text: String,
    pub level: i64,
    pub health: i64,
    pub might: i64,
    pub defense: i64,
    pub magic: i64,
    pub resistance: i64,
    pub agility: i64,
    #[serde(rename = "experienceReward")]
    pub experience_reward: i64,
    #[serde(rename = "creatureType")]
    pub creature_type: String,
}
