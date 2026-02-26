pub mod dual_wield;
pub mod enchant_weapon;
pub mod fire_strike;
pub mod ice_strike;
pub mod overdrive;

use std::fmt::Display;

use anyhow::Error;
use serde::{Deserialize, Serialize};

use crate::game::ability_effects::{
    AbilityEffect, ActiveEffectType, EffectType, PassiveEffectType,
};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub enum AbilityType {
    Combat,
    Passive,
    NonCombat,
}

impl Display for AbilityType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AbilityType::Combat => write!(f, "combat"),
            AbilityType::Passive => write!(f, "passive"),
            AbilityType::NonCombat => write!(f, "non_combat"),
        }
    }
}

/// Active buff tracking for noncombat abilities
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ActiveBuff {
    pub ability_id: String,
    pub ability_name: String,
    pub remaining_adventures: i64,
}

/// Ability represents a skill or spell that characters can use
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Ability {
    pub id: &'static str, // Unique identifier for the ability
    pub name: &'static str,
    pub description: &'static str,
    // Ability type: "combat" = shows in combat action bar, "passive" = hidden/automatic, "noncombat" = buff used before combat
    pub ability_type: AbilityType,
    pub effects: &'static [AbilityEffect],
    pub mana_cost: Option<i64>,
    pub cooldown: Option<i64>,
    // For noncombat abilities - how many adventures the buff lasts
    pub duration: Option<i64>,
}

impl Ability {
    /// Check if ability has a specific passive effect
    pub fn has_passive(&self, passive_type: PassiveEffectType) -> bool {
        self.effects
            .iter()
            .any(|e| e.effect_type == EffectType::Passive && e.passive_type == Some(passive_type))
    }

    /// Get the weapon enchantment type if this ability provides one
    /// Returns Some(PassiveEffectType::EnchantWeaponFire), etc., or None
    pub fn get_enchantment_type(&self) -> Option<PassiveEffectType> {
        for effect in self.effects {
            if effect.effect_type == EffectType::Passive {
                if let Some(passive_type) = effect.passive_type {
                    match passive_type {
                        PassiveEffectType::EnchantWeaponFire
                        | PassiveEffectType::EnchantWeaponFrost
                        | PassiveEffectType::EnchantWeaponLightning => {
                            return Some(passive_type);
                        }
                        _ => {}
                    }
                }
            }
        }
        None
    }

    /// Check if this ability provides weapon enchantment damage
    /// (has both passive enchantment effect and active damage effect)
    pub fn provides_enchantment_damage(&self) -> bool {
        let has_enchant_passive = self.effects.iter().any(|e| {
            e.effect_type == EffectType::Passive
                && matches!(
                    e.passive_type,
                    Some(PassiveEffectType::EnchantWeaponFire)
                        | Some(PassiveEffectType::EnchantWeaponFrost)
                        | Some(PassiveEffectType::EnchantWeaponLightning)
                )
        });

        let has_damage_effect = self.effects.iter().any(|e| {
            e.effect_type == EffectType::Active && e.active_type == Some(ActiveEffectType::Damage)
        });

        has_enchant_passive && has_damage_effect
    }
}

/// CharacterAbility tracks which abilities a character has unlocked
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CharacterAbility {
    pub character_id: String,
    pub ability_id: String,
    pub unlocked_at_level: i64,
}

static ALL_ABILITIES: &[&Ability] = &[
    &fire_strike::FIRE_STRIKE,
    &dual_wield::DUAL_WIELD,
    &ice_strike::ICE_STRIKE,
    &enchant_weapon::ENCHANT_WEAPON,
    &overdrive::OVERDRIVE,
];

pub fn get_ability_by_id(id: &str) -> Result<&'static Ability, Error> {
    Ok(ALL_ABILITIES
        .iter()
        .find(|&&ability| ability.id == id)
        .copied()
        .ok_or_else(|| {
            Error::msg(format!(
                "Ability with id '{}' not found. Available abilities: {:?}",
                id,
                ALL_ABILITIES.iter().map(|a| a.id).collect::<Vec<_>>()
            ))
        })?)
}
