use serde::{Deserialize, Serialize};

/// Location represents a game location/area
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Location {
    pub id: String,
    pub name: String,
    pub description: String,
    #[serde(rename = "minLevel")]
    pub min_level: i64,
    #[serde(rename = "maxLevel")]
    pub max_level: i64,
    pub tier: i64,
    pub enabled: bool,
    #[serde(rename = "createdAt")]
    pub created_at: String,
}

/// DTO for creating a location (no id or createdAt)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateLocationRequest {
    pub name: String,
    pub description: String,
    #[serde(rename = "minLevel")]
    pub min_level: i64,
    #[serde(rename = "maxLevel")]
    pub max_level: i64,
    pub tier: i64,
    pub enabled: bool,
}

/// LocationCreature associates creatures with locations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LocationCreature {
    #[serde(rename = "locationId")]
    pub location_id: String,
    #[serde(rename = "creatureId")]
    pub creature_id: String,
    #[serde(rename = "spawnRate")]
    pub spawn_rate: i64, // 1-100, probability of encountering this creature
}

/// LocationAdventure associates adventures with locations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LocationAdventure {
    #[serde(rename = "locationId")]
    pub location_id: String,
    #[serde(rename = "adventureId")]
    pub adventure_id: String,
    #[serde(rename = "spawnRate")]
    pub spawn_rate: i64, // 1-100, probability of encountering this adventure
}
