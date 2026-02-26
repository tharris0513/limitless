use crate::game::{
    abilities::{Ability, AbilityType},
    ability_effects::{AbilityEffect, EffectType, PassiveEffectType},
};

pub const ENCHANT_WEAPON: Ability = Ability {
    id: "enchant_weapon",
    name: "Enchant Weapon",
    description: "When striking an enemy with one of your elemental strike abilities, you imbue your weapon with that element's energy. If you are dual wielding, the second strike will enchant your offhand weapon.",
    ability_type: AbilityType::Passive,
    mana_cost: None,
    cooldown: None,
    duration: None,
    effects: &[AbilityEffect {
        id: "enchant_passive",
        effect_type: EffectType::Passive,
        active_type: None,
        passive_type: Some(PassiveEffectType::EnchantWeapon),
        formula: None,
        combat_description: None,
        stat_modifiers: None,
    }],
};
