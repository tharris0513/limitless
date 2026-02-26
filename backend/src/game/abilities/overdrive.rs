use crate::game::{
    abilities::{Ability, AbilityType},
    ability_effects::{AbilityEffect, EffectType, ModifierType, StatModifier},
    Statistic,
};

pub const OVERDRIVE: Ability = Ability {
    id: "overdrive",
    name: "Overdrive",
    description: "Drop your defenses to zero in return for a boost to might, magic, and agility.",
    ability_type: AbilityType::NonCombat,
    effects: &[AbilityEffect {
        id: "overdrive_buff",
        effect_type: EffectType::Passive,
        active_type: None,
        formula: None,
        combat_description: None,
        passive_type: None,
        stat_modifiers: Some(&[
            StatModifier {
                stat: Statistic::Might,
                modifier_type: ModifierType::Percentage,
                value: 20.0,
            },
            StatModifier {
                stat: Statistic::Magic,
                modifier_type: ModifierType::Percentage,
                value: 20.0,
            },
            StatModifier {
                stat: Statistic::Agility,
                modifier_type: ModifierType::Percentage,
                value: 10.0,
            },
            StatModifier {
                stat: Statistic::Defense,
                modifier_type: ModifierType::Set,
                value: 0.0,
            },
            StatModifier {
                stat: Statistic::Resistance,
                modifier_type: ModifierType::Set,
                value: 0.0,
            },
        ]),
    }],
    mana_cost: Some(10),
    cooldown: None,
    duration: Some(10),
};
