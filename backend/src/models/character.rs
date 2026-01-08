use serde::{Deserialize, Serialize};

/// Character stats structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CharacterStats {
    pub might: i64,      // Physical power
    pub defense: i64,    // Physical defense
    pub magic: i64,      // Magical power
    pub resistance: i64, // Magical defense
    pub agility: i64,    // Speed
    pub adventures: i64,
    pub health: i64,
    #[serde(rename = "maxHealth")]
    pub max_health: i64,
    pub mana: i64,
    #[serde(rename = "maxMana")]
    pub max_mana: i64,
}

/// Character represents a game character that belongs to a user
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Character {
    pub id: String, // Character UUID
    #[serde(rename = "userId")]
    pub user_id: String, // Foreign key to User
    pub name: String, // Character name (chosen by player)
    #[serde(rename = "classId")]
    pub class_id: String, // Character's class
    pub level: i64,
    pub experience: i64,
    #[serde(rename = "experienceToNext")]
    pub experience_to_next: i64,
    pub stats: CharacterStats,
    pub location: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "gameState")]
    pub game_state: Option<String>, // JSON string storing current game state
    #[serde(rename = "createdAt")]
    pub created_at: String,
    #[serde(rename = "lastPlayed")]
    pub last_played: String,
}

/// Class represents a character class with base stats
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Class {
    pub id: String,
    pub name: String,
    pub description: String,
    #[serde(rename = "startingMight")]
    pub starting_might: i64,
    #[serde(rename = "startingDefense")]
    pub starting_defense: i64,
    #[serde(rename = "startingMagic")]
    pub starting_magic: i64,
    #[serde(rename = "startingResistance")]
    pub starting_resistance: i64,
    #[serde(rename = "startingAgility")]
    pub starting_agility: i64,
    #[serde(rename = "startingHealth")]
    pub starting_health: i64,
    #[serde(rename = "startingMana")]
    pub starting_mana: i64,
}

/// Ability represents a skill or spell that characters can use
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Ability {
    pub id: String,
    pub name: String,
    pub description: String,
    #[serde(rename = "abilityType")]
    pub ability_type: String, // 'active' or 'passive'
    #[serde(rename = "manaCost")]
    pub mana_cost: i64,
    pub cooldown: i64,

    // Formula-based calculations (for active abilities)
    #[serde(rename = "damageFormula", skip_serializing_if = "Option::is_none")]
    pub damage_formula: Option<String>, // e.g., "(might * 0.8) + 15"
    #[serde(rename = "healFormula", skip_serializing_if = "Option::is_none")]
    pub heal_formula: Option<String>, // e.g., "(magic * 1.2) + 20"
    #[serde(rename = "effectFormula", skip_serializing_if = "Option::is_none")]
    pub effect_formula: Option<String>, // For complex status effects

    // Passive ability effect tag (for passive abilities)
    #[serde(rename = "passiveEffect", skip_serializing_if = "Option::is_none")]
    pub passive_effect: Option<String>, // e.g., "dual_wield", "increased_crit", etc.
}

/// Available passive ability effects
pub const PASSIVE_EFFECTS: &[(&str, &str)] = &[
    ("dual_wield", "Dual Wield - Attack with both weapons"),
    (
        "increased_crit",
        "Increased Critical - +10% critical hit chance",
    ),
    ("lifesteal", "Lifesteal - Heal for 15% of damage dealt"),
    ("thorns", "Thorns - Reflect 20% of damage taken"),
    ("mana_regen", "Mana Regeneration - Restore 5 mana per turn"),
    (
        "health_regen",
        "Health Regeneration - Restore 10 health per turn",
    ),
    ("dodge_bonus", "Dodge Bonus - +15% chance to dodge attacks"),
    (
        "armor_pierce",
        "Armor Piercing - Ignore 25% of enemy defense",
    ),
    ("spell_amp", "Spell Amplification - +20% magic damage"),
    ("iron_skin", "Iron Skin - +15% physical damage reduction"),
    (
        "arcane_shield",
        "Arcane Shield - +15% magical damage reduction",
    ),
    ("berserker", "Berserker - +5% damage per 10% missing health"),
    (
        "first_strike",
        "First Strike - Always attack first in combat",
    ),
    (
        "counter_attack",
        "Counter Attack - 30% chance to attack when hit",
    ),
    ("vampiric_aura", "Vampiric Aura - Restore health on kill"),
];

/// ClassAbility maps abilities to classes with unlock levels
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClassAbility {
    pub class_id: String,
    pub ability_id: String,
    pub unlock_level: i64,
}

/// CharacterAbility tracks which abilities a character has unlocked
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CharacterAbility {
    pub character_id: String,
    pub ability_id: String,
    pub unlocked_at_level: i64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ability: Option<Ability>, // Joined ability data
}
