pub mod spellblade;

use anyhow::{Error, Result};
use serde::Serialize;

use crate::game::abilities::Ability;

/// Class represents a character class with base stats
#[derive(Debug, Clone, Copy, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Class {
    pub id: &'static str,
    pub name: &'static str,
    pub description: &'static str,
    pub starting_might: i64,
    pub starting_defense: i64,
    pub starting_magic: i64,
    pub starting_resistance: i64,
    pub starting_agility: i64,
    pub starting_health: i64,
    pub starting_mana: i64,
    pub ability_unlocks: &'static [(i64, &'static [&'static Ability])],
}

static ALL_CLASSES: &[&Class] = &[&spellblade::SPELLBLADE];

pub fn get_all_classes() -> Result<Vec<Class>> {
    Ok(ALL_CLASSES.iter().map(|&&class| class).collect())
}

pub fn get_class_by_id(id: &str) -> Result<&'static Class> {
    Ok(ALL_CLASSES
        .iter()
        .find(|&&class| class.id == id)
        .copied()
        .ok_or_else(|| {
            Error::msg(format!(
                "Class with id '{}' not found. Available classes: {:?}",
                id,
                ALL_CLASSES.iter().map(|c| c.id).collect::<Vec<_>>()
            ))
        })?)
}

pub fn get_class_abilities_with_levels(class_id: &str) -> Result<Vec<(&'static Ability, i64)>> {
    let class = get_class_by_id(class_id)?;
    Ok(class
        .ability_unlocks
        .iter()
        .flat_map(|(level, abilities)| abilities.iter().map(move |a| (*a, *level)))
        .collect())
}
