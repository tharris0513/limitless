use crate::game::{
    abilities::{Ability, AbilityType},
    ability_effects::dual_wield_passive::DUAL_WIELD_PASSIVE,
};

pub const DUAL_WIELD: Ability = Ability {
    id: "dual_wield",
    name: "Dual Wield",
    description:
        "Wield a weapon in each hand, allowing you to strike twice with a normal attack in combat.",
    ability_type: AbilityType::Passive,
    effects: &[DUAL_WIELD_PASSIVE],
    mana_cost: None,
    cooldown: None,
    duration: None,
};
