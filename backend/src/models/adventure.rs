use serde::{Deserialize, Serialize};

/// Adventure represents a noncombat encounter/event
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Adventure {
    pub id: String,
    pub name: String,
    pub description: String,
    #[serde(rename = "requiredLevel")]
    pub required_level: i64,
    #[serde(rename = "adventureType")]
    pub adventure_type: String, // 'puzzle', 'dialogue', 'exploration', etc.
    #[serde(rename = "experienceReward")]
    pub experience_reward: i64,
    #[serde(rename = "createdAt")]
    pub created_at: String,
}
