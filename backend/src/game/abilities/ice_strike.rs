use crate::game::{
    abilities::{Ability, AbilityType},
    ability_effects::{AbilityEffect, ActiveEffectType, EffectType, PassiveEffectType},
};

pub const ICE_STRIKE: Ability = Ability {
    id: "ice_strike",
    name: "Ice Strike",
    description:
        "Imbue your weapon with the power of ice, striking your enemy for physical and ice damage.",
    ability_type: AbilityType::Combat,
    effects: &[
        AbilityEffect {
            id: "ice_strike_damage",
            effect_type: EffectType::Active,
            active_type: Some(ActiveEffectType::Damage),
            formula: Some("(might * 1.0) + (magic * 1.0)"),
            combat_description: Some(
                "You strike the enemy with an icy blow, dealing **${damage}** damage!",
            ),
            passive_type: None,
            stat_modifiers: None,
        },
        AbilityEffect {
            id: "ice_enchant_passive",
            effect_type: EffectType::Passive,
            active_type: None,
            passive_type: Some(PassiveEffectType::EnchantWeaponFrost),
            formula: None,
            combat_description: None,
            stat_modifiers: None,
        },
    ],
    mana_cost: Some(2),
    cooldown: None,
    duration: None,
};
