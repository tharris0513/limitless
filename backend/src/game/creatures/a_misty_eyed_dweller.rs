use crate::game::creatures::Creature;

pub const A_MISTY_EYED_DWELLER: Creature = Creature {
    id: "a_misty_eyed_dweller",
    name: "A Misty-Eyed Dweller",
    introduction_text: "A figure barely resembling a humanoid emerges from the mist, its eyes clouded over with an eerie fog. You ready yourself for combat.",
    level: 3,
    max_health: 100,
    might: 10,
    defense: 5,
    magic: 10,
    resistance: 5,
    agility: 5,
    experience_reward: 800,
    creature_type: "horror",
    attack_description: "The abominable creature swipes at you with its claws, dealing ${damage} damage!",
};
