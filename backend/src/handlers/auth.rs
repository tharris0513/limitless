use crate::jwt::JwtService;
use crate::models::{
    AuthResponse, DiscordTokenResponse, DiscordUser, OAuthCallbackRequest, OAuthUrlResponse,
};
use crate::repository::UserRepository;
use axum::{
    extract::{Query, State},
    http::{header::SET_COOKIE, StatusCode},
    response::{IntoResponse, Redirect, Response},
    Json,
};
use serde_json::json;
use std::{collections::HashMap, env, sync::Arc};

pub async fn get_discord_auth_url() -> Json<OAuthUrlResponse> {
    let client_id = env::var("DISCORD_CLIENT_ID").expect("DISCORD_CLIENT_ID must be set in .env");
    let redirect_uri =
        env::var("DISCORD_REDIRECT_URI").expect("DISCORD_REDIRECT_URI must be set in .env");

    let state = uuid::Uuid::new_v4().to_string();
    let scope = "identify email";

    let auth_url = format!(
        "https://discord.com/api/oauth2/authorize?client_id={}&redirect_uri={}&response_type=code&scope={}&state={}",
        client_id,
        urlencoding::encode(&redirect_uri),
        urlencoding::encode(scope),
        state
    );

    tracing::info!("Generated Discord OAuth URL for authentication");

    Json(OAuthUrlResponse { auth_url, state })
}

// JSON-based Discord callback for API-driven auth flow
pub async fn discord_callback_json(
    State(repo): State<Arc<UserRepository>>,
    Query(params): Query<OAuthCallbackRequest>,
) -> Result<Json<AuthResponse>, Response> {
    tracing::info!("Discord OAuth callback (JSON) received");

    // Exchange code for access token
    let token_response = match exchange_code_for_token(&params.code).await {
        Ok(response) => {
            tracing::info!("Successfully exchanged code for token");
            response
        }
        Err(e) => {
            tracing::error!("Failed to exchange code for token: {:?}", e);
            let error = json!({
                "error": "Failed to authenticate with Discord",
                "details": "Authentication process failed"
            });
            return Err((StatusCode::INTERNAL_SERVER_ERROR, Json(error)).into_response());
        }
    };

    // Get user info from Discord
    let discord_user = match get_discord_user(&token_response.access_token).await {
        Ok(user) => {
            tracing::info!(
                "Successfully retrieved Discord user info: {}",
                user.username
            );
            user
        }
        Err(e) => {
            tracing::error!("Failed to get Discord user info: {:?}", e);
            let error = json!({
                "error": "Failed to get user information from Discord",
                "details": format!("User info request failed: {}", e)
            });
            return Err((StatusCode::INTERNAL_SERVER_ERROR, Json(error)).into_response());
        }
    };

    // Create or find user in database
    let discord_name = format!("{}#{}", discord_user.username, discord_user.discriminator);
    let user = match repo
        .find_or_create_by_discord_id(&discord_user.id, &discord_name)
        .await
    {
        Ok(user) => {
            tracing::info!("Successfully found/created user: {}", user.discord_name);
            user
        }
        Err(e) => {
            tracing::error!("Failed to find/create user: {:?}", e);
            let error = json!({
                "error": "Failed to create user account",
                "details": format!("Database error: {}", e)
            });
            return Err((StatusCode::INTERNAL_SERVER_ERROR, Json(error)).into_response());
        }
    };

    // Generate JWT token for the user
    let auth_token = match JwtService::generate_token_with_repo(&user, &repo).await {
        Ok(token) => token,
        Err(e) => {
            tracing::error!("Failed to generate JWT token: {:?}", e);
            let error = json!({
                "error": "Failed to generate authentication token",
                "details": format!("JWT error: {}", e)
            });
            return Err((StatusCode::INTERNAL_SERVER_ERROR, Json(error)).into_response());
        }
    };

    tracing::info!(
        "Successfully authenticated Discord user (JSON): {}",
        user.discord_name
    );

    let response = AuthResponse {
        token: auth_token,
        user,
    };

    Ok(Json(response))
}

pub async fn discord_callback(
    State(repo): State<Arc<UserRepository>>,
    Query(params): Query<OAuthCallbackRequest>,
) -> Result<Response, Response> {
    tracing::info!("Discord OAuth callback received");

    // Exchange code for access token
    let token_response = match exchange_code_for_token(&params.code).await {
        Ok(response) => {
            tracing::info!("Successfully exchanged code for token");
            response
        }
        Err(e) => {
            tracing::error!("Failed to exchange code for token: {:?}", e);
            let error = json!({
                "error": "Failed to authenticate with Discord",
                "details": "Authentication process failed"
            });
            return Err((StatusCode::INTERNAL_SERVER_ERROR, Json(error)).into_response());
        }
    };

    // Get user info from Discord
    let discord_user = match get_discord_user(&token_response.access_token).await {
        Ok(user) => {
            tracing::info!("Successfully retrieved Discord user: {}", user.username);
            user
        }
        Err(e) => {
            tracing::error!("Failed to get Discord user info: {:?}", e);
            let error = json!({
                "error": "Failed to get user information",
                "details": format!("User info request failed: {}", e)
            });
            return Err((StatusCode::INTERNAL_SERVER_ERROR, Json(error)).into_response());
        }
    };

    // Create or find user in database
    let discord_name = format!("{}#{}", discord_user.username, discord_user.discriminator);
    let user = match repo
        .find_or_create_by_discord_id(&discord_user.id, &discord_name)
        .await
    {
        Ok(user) => {
            tracing::info!("Successfully found/created user: {}", user.discord_name);
            user
        }
        Err(e) => {
            tracing::error!("Failed to find/create user: {:?}", e);
            let error = json!({
                "error": "Failed to create user account",
                "details": format!("Database error: {}", e)
            });
            return Err((StatusCode::INTERNAL_SERVER_ERROR, Json(error)).into_response());
        }
    };

    // Generate JWT token for the user
    let auth_token = match JwtService::generate_token_with_repo(&user, &repo).await {
        Ok(token) => token,
        Err(e) => {
            tracing::error!("Failed to generate JWT token: {:?}", e);
            let error = json!({
                "error": "Failed to generate authentication token",
                "details": format!("JWT error: {}", e)
            });
            return Err((StatusCode::INTERNAL_SERVER_ERROR, Json(error)).into_response());
        }
    };

    tracing::info!(
        "Successfully authenticated Discord user: {}",
        user.discord_name
    );

    // Set JWT token in HTTP-only cookie and redirect to clean URL
    let cookie = format!(
        "auth_token={}; Path=/; HttpOnly; Secure; SameSite=Lax; Max-Age=86400",
        auth_token
    );

    let frontend_url =
        env::var("FRONTEND_URL").unwrap_or_else(|_| "http://localhost:5173".to_string());
    let mut response = Redirect::to(&frontend_url).into_response();
    response
        .headers_mut()
        .insert(SET_COOKIE, cookie.parse().unwrap());

    Ok(response)
}

async fn exchange_code_for_token(
    code: &str,
) -> Result<DiscordTokenResponse, Box<dyn std::error::Error>> {
    let client = reqwest::Client::new();

    let client_id =
        env::var("DISCORD_CLIENT_ID").unwrap_or_else(|_| "your_discord_client_id_here".to_string());
    let client_secret = env::var("DISCORD_CLIENT_SECRET")
        .unwrap_or_else(|_| "your_discord_client_secret_here".to_string());
    let redirect_uri = env::var("DISCORD_REDIRECT_URI")
        .unwrap_or_else(|_| "http://localhost:8080/api/auth/discord/callback".to_string());

    let mut params = HashMap::new();
    params.insert("client_id", client_id.as_str());
    params.insert("client_secret", client_secret.as_str());
    params.insert("grant_type", "authorization_code");
    params.insert("code", code);
    params.insert("redirect_uri", redirect_uri.as_str());

    tracing::info!("Exchanging Discord authorization code for access token");

    let response = client
        .post("https://discord.com/api/oauth2/token")
        .header("Content-Type", "application/x-www-form-urlencoded")
        .form(&params)
        .send()
        .await?;

    let status = response.status();
    let response_text = response.text().await?;

    tracing::info!("Discord token exchange completed with status: {}", status);

    if !status.is_success() {
        return Err(format!("Discord API error: {} - {}", status, response_text).into());
    }

    let token_response: DiscordTokenResponse =
        serde_json::from_str(&response_text).map_err(|e| {
            format!(
                "Failed to parse Discord token response: {} - Response: {}",
                e, response_text
            )
        })?;

    Ok(token_response)
}

async fn get_discord_user(access_token: &str) -> Result<DiscordUser, Box<dyn std::error::Error>> {
    let client = reqwest::Client::new();

    let response = client
        .get("https://discord.com/api/users/@me")
        .header("Authorization", format!("Bearer {}", access_token))
        .send()
        .await?
        .json::<DiscordUser>()
        .await?;

    Ok(response)
}

pub async fn get_auth_from_cookie(
    headers: axum::http::HeaderMap,
) -> Result<Json<AuthResponse>, StatusCode> {
    // Extract JWT from HTTP-only cookie
    let cookie_header = headers
        .get("cookie")
        .and_then(|h| h.to_str().ok())
        .ok_or(StatusCode::UNAUTHORIZED)?;

    // Parse cookies to find auth_token
    let token = cookie_header
        .split(';')
        .find_map(|cookie| {
            let cookie = cookie.trim();
            if cookie.starts_with("auth_token=") {
                Some(&cookie[11..]) // Skip "auth_token="
            } else {
                None
            }
        })
        .ok_or(StatusCode::UNAUTHORIZED)?;

    // Verify the token
    let claims = JwtService::verify_token(token).map_err(|_| StatusCode::UNAUTHORIZED)?;

    // Get user from database (optional, for returning user data)
    let response = AuthResponse {
        token: token.to_string(),
        user: crate::models::User {
            id: claims.sub.clone(),
            discord_id: claims.discord_id.clone(),
            discord_name: "User".to_string(), // We could get this from DB if needed
            username: None,
            date_of_birth: None,
            admin: claims.admin,
            banned: false,
            created_at: chrono::Utc::now().to_rfc3339(),
        },
    };

    Ok(Json(response))
}

pub async fn get_auth_status(headers: axum::http::HeaderMap) -> Json<serde_json::Value> {
    if let Some(auth_header) = headers.get("authorization").and_then(|h| h.to_str().ok()) {
        if let Some(token) = JwtService::extract_token_from_auth_header(auth_header) {
            if JwtService::verify_token(token).is_ok() {
                return Json(json!({"authenticated": true}));
            }
        }
    }
    Json(json!({"authenticated": false}))
}

pub async fn verify_token(
    State(repo): State<Arc<UserRepository>>,
    headers: axum::http::HeaderMap,
) -> Result<Json<AuthResponse>, StatusCode> {
    let auth_header = headers
        .get("authorization")
        .and_then(|header| header.to_str().ok())
        .ok_or(StatusCode::UNAUTHORIZED)?;

    let token =
        JwtService::extract_token_from_auth_header(auth_header).ok_or(StatusCode::UNAUTHORIZED)?;

    let claims = JwtService::verify_token(token).map_err(|_| StatusCode::UNAUTHORIZED)?;

    let user = repo
        .find_by_id(&claims.sub)
        .await
        .map_err(|_| StatusCode::NOT_FOUND)?;

    let response = AuthResponse {
        token: token.to_string(),
        user,
    };

    Ok(Json(response))
}

pub async fn logout() -> Response {
    // Clear the HTTP-only cookie by setting it with Max-Age=0
    let clear_cookie = "auth_token=; Path=/; HttpOnly; Secure; SameSite=Lax; Max-Age=0";

    tracing::info!("User logged out");

    let mut response = Json(json!({"message": "Successfully logged out"})).into_response();
    response.headers_mut().insert(
        axum::http::header::SET_COOKIE,
        clear_cookie.parse().unwrap(),
    );

    response
}
