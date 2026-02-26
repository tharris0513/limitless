use serde::Serialize;

use crate::game::Statistic;

pub mod dual_wield_passive;

#[derive(Debug, Clone, Copy, Serialize, PartialEq, Eq)]
pub enum EffectType {
    Active,
    Passive,
}

#[derive(Debug, Clone, Copy, Serialize, PartialEq, Eq)]
pub enum ActiveEffectType {
    Damage,
}

/// Passive effect type - specific passive abilities
#[derive(Debug, Clone, Copy, Serialize, PartialEq, Eq)]
pub enum PassiveEffectType {
    DualWield,
    EnchantWeapon,
    EnchantWeaponFire,
    EnchantWeaponFrost,
    EnchantWeaponLightning,
}

impl PassiveEffectType {
    pub fn enchantment_element(&self) -> Option<&'static str> {
        match self {
            PassiveEffectType::EnchantWeaponFire => Some("fire"),
            PassiveEffectType::EnchantWeaponFrost => Some("frost"),
            PassiveEffectType::EnchantWeaponLightning => Some("lightning"),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Copy, Serialize, PartialEq, Eq)]
pub enum ModifierType {
    Percentage,
    Set,
}

/// Stat modifier for passive effects
#[derive(Debug, Clone, Serialize)]
pub struct StatModifier {
    pub stat: Statistic,
    pub value: f64,
    pub modifier_type: ModifierType,
}

/// Represents a single effect within an ability
#[derive(Debug, Clone, Serialize)]
pub struct AbilityEffect {
    pub id: &'static str, // Unique ID for this effect within the ability
    pub effect_type: EffectType,
    pub active_type: Option<ActiveEffectType>,
    pub combat_description: Option<&'static str>, // Description for active effects
    pub formula: Option<&'static str>,            // The calculation formula
    pub passive_type: Option<PassiveEffectType>,
    pub stat_modifiers: Option<&'static [StatModifier]>,
}
