use serde::{Deserialize, Serialize};

/// User represents a Discord account holder
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

/// Discord API user response
#[derive(Debug, Deserialize)]
pub struct DiscordUser {
    pub id: String,
    pub username: String,
    pub discriminator: String,
}

/// Discord OAuth token response
#[derive(Debug, Deserialize)]
pub struct DiscordTokenResponse {
    pub access_token: String,
}

/// JWT claims for authenticated users
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Claims {
    pub sub: String,        // User ID (subject)
    pub discord_id: String, // Discord user ID
    pub admin: bool,        // Admin status
    pub exp: usize,         // Expiration timestamp
    pub gen: usize,         // Token generation (for invalidating all tokens)
}

/// OAuth URL response with auth URL and state token
#[derive(Debug, Serialize)]
pub struct OAuthUrlResponse {
    pub auth_url: String,
    pub state: String,
}

/// OAuth callback request with authorization code
#[derive(Debug, Deserialize)]
pub struct OAuthCallbackRequest {
    pub code: String,
}

/// Authentication response with JWT token and user data
#[derive(Debug, Serialize)]
pub struct AuthResponse {
    pub token: String,
    pub user: User,
}
