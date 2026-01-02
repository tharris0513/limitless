use serde::{Deserialize, Serialize};

// User represents a Discord account holder
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct User {
    pub id: String,           // UUID
    pub discord_id: String,   // Discord ID from OAuth
    pub discord_name: String, // Discord username with discriminator
    #[serde(skip_serializing_if = "Option::is_none")]
    pub username: Option<String>, // User's chosen username
    #[serde(skip_serializing_if = "Option::is_none")]
    pub date_of_birth: Option<String>, // User's date of birth (YYYY-MM-DD)
    pub admin: bool,          // Admin flag
    pub banned: bool,         // Banned flag
    pub created_at: String,   // When user first logged in
}

// Character stats structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CharacterStats {
    pub might: i64,      // Physical power
    pub defense: i64,    // Physical defense
    pub magic: i64,      // Magical power
    pub resistance: i64, // Magical defense
    pub agility: i64,    // Speed
    pub adventures: i64,
}

// Character represents a game character that belongs to a user
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Character {
    pub id: String, // Character UUID
    #[serde(rename = "userId")]
    pub user_id: String, // Foreign key to User
    pub name: String, // Character name (chosen by player)
    #[serde(rename = "classId")]
    pub class_id: String, // Character's class
    pub level: i64,
    pub health: i64,
    #[serde(rename = "maxHealth")]
    pub max_health: i64,
    pub mana: i64,
    #[serde(rename = "maxMana")]
    pub max_mana: i64,
    pub experience: i64,
    #[serde(rename = "experienceToNext")]
    pub experience_to_next: i64,
    pub stats: CharacterStats, // Will be loaded from character_stats table
    pub location: String,
    #[serde(rename = "createdAt")]
    pub created_at: String,
    #[serde(rename = "lastPlayed")]
    pub last_played: String,
}

// Class represents a character class with base stats
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

// Ability represents a skill or spell that characters can use
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

    // Formula-based calculations
    #[serde(rename = "damageFormula", skip_serializing_if = "Option::is_none")]
    pub damage_formula: Option<String>, // e.g., "(might * 0.8) + 15"
    #[serde(rename = "healFormula", skip_serializing_if = "Option::is_none")]
    pub heal_formula: Option<String>, // e.g., "(magic * 1.2) + 20"
    #[serde(rename = "effectFormula", skip_serializing_if = "Option::is_none")]
    pub effect_formula: Option<String>, // For complex status effects
}

// ClassAbility maps abilities to classes with unlock levels
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClassAbility {
    pub class_id: String,
    pub ability_id: String,
    pub unlock_level: i64,
}

// CharacterAbility tracks which abilities a character has unlocked
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CharacterAbility {
    pub character_id: String,
    pub ability_id: String,
    pub unlocked_at_level: i64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ability: Option<Ability>, // Joined ability data
}

#[derive(Debug, Deserialize)]
pub struct DiscordUser {
    pub id: String,
    pub username: String,
    pub discriminator: String,
}

#[derive(Debug, Deserialize)]
pub struct DiscordTokenResponse {
    pub access_token: String,
}

// Request/Response DTOs
#[derive(Debug, Deserialize)]
pub struct OAuthCallbackRequest {
    pub code: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Claims {
    pub sub: String,        // User ID (subject)
    pub discord_id: String, // Discord user ID
    pub admin: bool,        // Admin status
    pub exp: usize,         // Expiration timestamp
}

#[derive(Debug, Serialize)]
pub struct OAuthUrlResponse {
    pub auth_url: String,
    pub state: String,
}

#[derive(Debug, Serialize)]
pub struct AuthResponse {
    pub token: String,
    pub user: User,
}
