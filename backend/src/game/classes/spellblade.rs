use crate::game::{
    abilities::{
        dual_wield::DUAL_WIELD, enchant_weapon::ENCHANT_WEAPON, fire_strike::FIRE_STRIKE,
        ice_strike::ICE_STRIKE, overdrive::OVERDRIVE,
    },
    classes::Class,
};

pub const SPELLBLADE: Class = Class {
    id: "spellblade",
    name: "Spellblade",
    description: "A versatile warrior who combines martial prowess with arcane magic.",
    starting_might: 12,
    starting_defense: 8,
    starting_magic: 12,
    starting_resistance: 8,
    starting_agility: 10,
    starting_health: 80,
    starting_mana: 20,
    ability_unlocks: &[
        (1, &[&FIRE_STRIKE]),
        (2, &[&DUAL_WIELD]),
        (3, &[&ICE_STRIKE]),
        (4, &[&ENCHANT_WEAPON]),
        (7, &[&OVERDRIVE]),
    ],
};
