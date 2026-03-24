use crate::game::{
    abilities::{Ability, AbilityType},
    ability_effects::{AbilityEffect, ActiveEffectType, EffectType, PassiveEffectType},
};

pub const LIGHTNING_STRIKE: Ability = Ability {
    id: "lightning_strike",
    name: "Lightning Strike",
    description:
        "Imbue your weapon with the power of lightning, striking your enemy for physical and lightning damage.",
    ability_type: AbilityType::Combat,
    effects: &[AbilityEffect {
        id: "lightning_strike_damage",
        effect_type: EffectType::Active,
        active_type: Some(ActiveEffectType::Damage),
        formula: Some("(might * 1.0 - enemy_defense) + (magic * 1.0 - enemy_resistance)"),
        combat_description: Some("You strike the enemy with a fulminating blow, dealing **${damage}** damage!"),
        passive_type: None,
        stat_modifiers: None,
    }, AbilityEffect {
        id: "lightning_enchant_passive",
        effect_type: EffectType::Passive,
        active_type: None,
        passive_type: Some(PassiveEffectType::EnchantWeaponLightning),
        formula: None,
        combat_description: None,
        stat_modifiers: None,
    },],
    mana_cost: Some(2),
    cooldown: None,
    duration: None,
};
