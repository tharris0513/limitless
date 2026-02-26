use anyhow::Result;
use serde::Serialize;

use crate::game::{creatures::Creature, locations::mistlands::MISTLANDS};

pub mod mistlands;

/// Location represents a game location/area
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Location {
    pub id: &'static str,
    pub name: &'static str,
    pub description: &'static str,
    pub min_level: i64,
    pub max_level: i64,
    pub tier: i64,
    pub enabled: bool,
    pub creatures: &'static [(Creature, i64)],
    pub adventures: &'static [(LocationAdventure, i64)],
}

/// LocationAdventure associates adventures with locations
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LocationAdventure {
    pub location_id: &'static str,
    pub adventure_id: &'static str,
    pub spawn_rate: i64, // 1-100, probability of encountering this adventure
}

pub const ALL_LOCATIONS: &[Location] = &[MISTLANDS];

pub fn get_all_locations() -> Result<Vec<Location>> {
    Ok(ALL_LOCATIONS.to_vec())
}

pub fn get_location_by_id(location_id: &str) -> Result<Location> {
    ALL_LOCATIONS
        .iter()
        .find(|loc| loc.id == location_id)
        .cloned()
        .ok_or_else(|| anyhow::anyhow!("Location not found: {}", location_id))
}

pub fn get_location_creatures(location_id: &str) -> Result<Vec<(Creature, i64)>> {
    Ok(get_location_by_id(location_id)?.creatures.to_vec())
}

pub fn get_location_adventures(location_id: &str) -> Result<Vec<(LocationAdventure, i64)>> {
    Ok(get_location_by_id(location_id)?.adventures.to_vec())
}
