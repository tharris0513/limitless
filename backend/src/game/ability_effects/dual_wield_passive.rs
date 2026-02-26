use crate::game::ability_effects::{AbilityEffect, EffectType, PassiveEffectType};

pub const DUAL_WIELD_PASSIVE: AbilityEffect = AbilityEffect {
    id: "dual_wield_passive",
    effect_type: EffectType::Passive,
    active_type: None,
    combat_description: None,
    formula: None,
    passive_type: Some(PassiveEffectType::DualWield),
    stat_modifiers: None,
};
