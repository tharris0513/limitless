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
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "gameState")]
    pub game_state: Option<String>, // JSON string storing current game state
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

// Location represents a game location/area
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Location {
    pub id: String,
    pub name: String,
    pub description: String,
    #[serde(rename = "minLevel")]
    pub min_level: i64,
    #[serde(rename = "maxLevel")]
    pub max_level: i64,
    pub tier: i64,
    pub enabled: bool,
    #[serde(rename = "createdAt")]
    pub created_at: String,
}
// DTO for creating a location (no id or createdAt)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateLocationRequest {
    pub name: String,
    pub description: String,
    #[serde(rename = "minLevel")]
    pub min_level: i64,
    #[serde(rename = "maxLevel")]
    pub max_level: i64,
    pub tier: i64,
    pub enabled: bool,
}
// Creature represents an enemy or NPC that can be encountered
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Creature {
    pub id: String,
    pub name: String,
    #[serde(rename = "introductionText")]
    pub introduction_text: String,
    pub level: i64,
    pub health: i64,
    pub might: i64,
    pub defense: i64,
    pub magic: i64,
    pub resistance: i64,
    pub agility: i64,
    #[serde(rename = "experienceReward")]
    pub experience_reward: i64,
    #[serde(rename = "creatureType")]
    pub creature_type: String, // 'beast', 'undead', 'humanoid', 'elemental', 'dragon', 'demon', 'horror'
    #[serde(rename = "createdAt")]
    pub created_at: String,
}

// Request DTO for creating a creature (without id and createdAt)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateCreatureRequest {
    pub name: String,
    #[serde(rename = "introductionText")]
    pub introduction_text: String,
    pub level: i64,
    pub health: i64,
    pub might: i64,
    pub defense: i64,
    pub magic: i64,
    pub resistance: i64,
    pub agility: i64,
    #[serde(rename = "experienceReward")]
    pub experience_reward: i64,
    #[serde(rename = "creatureType")]
    pub creature_type: String,
}

// Adventure represents a noncombat encounter/event
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Adventure {
    pub id: String,
    pub name: String,
    pub description: String,
    #[serde(rename = "requiredLevel")]
    pub required_level: i64,
    #[serde(rename = "adventureType")]
    pub adventure_type: String, // 'puzzle', 'dialogue', 'exploration', etc.
    #[serde(rename = "experienceReward")]
    pub experience_reward: i64,
    #[serde(rename = "createdAt")]
    pub created_at: String,
}

// LocationCreature associates creatures with locations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LocationCreature {
    #[serde(rename = "locationId")]
    pub location_id: String,
    #[serde(rename = "creatureId")]
    pub creature_id: String,
    #[serde(rename = "spawnRate")]
    pub spawn_rate: i64, // 1-100, probability of encountering this creature
}

// LocationAdventure associates adventures with locations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LocationAdventure {
    #[serde(rename = "locationId")]
    pub location_id: String,
    #[serde(rename = "adventureId")]
    pub adventure_id: String,
    #[serde(rename = "spawnRate")]
    pub spawn_rate: i64, // 1-100, probability of encountering this adventure
}

#[derive(Debug, Serialize)]
pub struct AuthResponse {
    pub token: String,
    pub user: User,
}
