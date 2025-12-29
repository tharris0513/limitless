use serde::{Deserialize, Serialize};

// User represents a Discord account holder
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct User {
    pub id: String,              // UUID
    pub discord_id: String,      // Discord ID from OAuth
    pub discord_name: String,    // Discord username with discriminator
    #[serde(skip_serializing_if = "Option::is_none")]
    pub username: Option<String>, // User's chosen username
    #[serde(skip_serializing_if = "Option::is_none")]
    pub date_of_birth: Option<String>, // User's date of birth (YYYY-MM-DD)
    pub admin: bool,             // Admin flag
    pub created_at: String,      // When user first logged in
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
    pub max_adventures: i64,
}

// Character represents a game character that belongs to a user
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Character {
    pub id: String,      // Character UUID
    pub user_id: String, // Foreign key to User
    pub name: String,    // Character name (chosen by player)
    pub level: i64,
    pub health: i64,
    pub max_health: i64,
    pub mana: i64,
    pub max_mana: i64,
    pub experience: i64,
    pub experience_to_next: i64,
    pub stats: CharacterStats, // Will be loaded from character_stats table
    pub location: String,
    pub created_at: String,
    pub last_played: String,
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
