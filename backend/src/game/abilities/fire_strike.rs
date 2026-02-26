use crate::game::{
    abilities::{Ability, AbilityType},
    ability_effects::{AbilityEffect, ActiveEffectType, EffectType, PassiveEffectType},
};

pub const FIRE_STRIKE: Ability = Ability {
    id: "fire_strike",
    name: "Fire Strike",
    description:
        "Imbue your weapon with the power of fire, striking your enemy for physical and fire damage.",
    ability_type: AbilityType::Combat,
    effects: &[AbilityEffect {
        id: "fire_strike_damage",
        effect_type: EffectType::Active,
        active_type: Some(ActiveEffectType::Damage),
        formula: Some("(might * 1.0) + (magic * 1.0)"),
        combat_description: Some("You strike the enemy with a fiery blow, dealing **${damage}** damage!"),
        passive_type: None,
        stat_modifiers: None,
    }, AbilityEffect {
        id: "fire_enchant_passive",
        effect_type: EffectType::Passive,
        active_type: None,
        passive_type: Some(PassiveEffectType::EnchantWeaponFire),
        formula: None,
        combat_description: None,
        stat_modifiers: None,
    },],
    mana_cost: Some(2),
    cooldown: None,
    duration: None,
};
