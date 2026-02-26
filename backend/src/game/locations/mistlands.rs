use crate::game::{creatures::a_misty_eyed_dweller::A_MISTY_EYED_DWELLER, locations::Location};

pub const MISTLANDS: Location = Location {
    id: "mistlands",
    name: "Mistlands",
    description: "...danger lurks between the trees...",
    min_level: 1,
    max_level: 10,
    tier: 1,
    enabled: true,
    creatures: &[(A_MISTY_EYED_DWELLER, 100)],
    adventures: &[],
};
