use axum::{
    middleware,
    routing::{get, patch, post},
    Router,
};
use std::sync::Arc;

use crate::{
    handlers::{admin, adventure, auth, character, health, location, player, user},
    middleware::{global_error_handler, jwt_auth_middleware},
    repository::UserRepository,
};

pub fn create_routes(repo: Arc<UserRepository>) -> Router {
    // Public routes (no authentication required)
    let public_routes = Router::new()
        .route("/auth/discord", get(auth::get_discord_auth_url))
        .route("/auth/discord/callback", get(auth::discord_callback))
        .route(
            "/auth/discord/callback/json",
            get(auth::discord_callback_json),
        )
        .route("/auth/cookie", get(auth::get_auth_from_cookie))
        .route("/auth/logout", post(auth::logout))
        .route("/auth/verify", get(auth::verify_token))
        .route("/auth/status", get(auth::get_auth_status))
        .route("/health", get(health::health_check))
        // Legacy routes for backward compatibility
        .route("/player", get(player::get_player))
        .route("/locations", get(location::get_locations))
        .route("/adventures/:id/start", post(adventure::start_adventure));

    // Protected routes (require JWT authentication)
    let protected_routes = Router::new()
        .route("/user", get(user::get_user))
        .route("/characters", get(character::get_user_characters))
        .route("/characters", post(character::create_character))
        .route("/characters/:id", patch(character::update_character))
        .route(
            "/characters/:id/last-played",
            patch(character::update_character_last_played),
        )
        .layer(middleware::from_fn(jwt_auth_middleware));

    // Admin-only routes
    // Note: jwt_auth_middleware only verifies the JWT token exists and is valid
    // The actual admin enforcement happens via the AdminClaims extractor in the handler
    // Handlers using AdminClaims(claims) will automatically reject non-admin users with 403
    let admin_routes = Router::new()
        .route("/admin/check", get(admin::check_admin_status))
        .route("/admin/users", get(admin::get_all_users))
        .layer(middleware::from_fn(jwt_auth_middleware));

    Router::new()
        .merge(public_routes)
        .merge(protected_routes)
        .merge(admin_routes)
        .layer(middleware::from_fn(global_error_handler))
        .with_state(repo)
}
