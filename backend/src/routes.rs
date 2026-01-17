use axum::{
    middleware,
    routing::{delete, get, patch, post},
    Router,
};
use std::sync::Arc;

use crate::{
    handlers::{
        admin, adventure, auth, character, chat, combat_action, health, location, player, user,
    },
    middleware::{global_error_handler, jwt_auth_middleware, maintenance_mode_middleware},
    repository::UserRepository,
};

pub fn create_routes(repo: Arc<UserRepository>) -> Router {
    // Create chat state
    let chat_state = Arc::new(chat::ChatState::new());

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
        .route("/maintenance", get(health::check_maintenance_mode))
        .route("/user/check-username", post(user::check_username_available))
        .route("/classes", get(character::get_classes))
        .route(
            "/classes/:class_id/abilities",
            get(character::get_class_abilities),
        )
        // Legacy routes for backward compatibility
        .route("/player", get(player::get_player))
        .route("/locations", get(location::get_locations))
        .route("/adventures/:id/start", post(adventure::start_adventure));

    // WebSocket route for chat (separate router with chat_state)
    let chat_routes = Router::new()
        .route("/chat/ws", get(chat::websocket_handler))
        .with_state(chat_state.clone());

    // Protected routes (require JWT authentication)
    let protected_routes = Router::new()
        .route("/user", get(user::get_user))
        .route("/user", patch(user::update_user))
        .route("/characters", get(character::get_user_characters))
        .route("/characters", post(character::create_character))
        .route("/characters/:id", get(character::get_character))
        .route("/characters/:id", patch(character::update_character))
        .route(
            "/characters/:id/last-played",
            patch(character::update_character_last_played),
        )
        .route(
            "/characters/:id/state",
            post(combat_action::save_game_state),
        )
        .route(
            "/characters/:id/state",
            delete(combat_action::clear_game_state),
        )
        .route("/characters/:id/flee", post(combat_action::flee_combat))
        .route(
            "/characters/:id/combat-action",
            post(combat_action::perform_combat_action),
        )
        .route("/characters/:id/rest", post(combat_action::rest_character))
        .route(
            "/characters/:id/abilities",
            get(character::get_character_abilities),
        )
        .route(
            "/characters/:id/experience",
            post(character::grant_experience),
        )
        .route("/locations/:id/visit", post(location::visit_location))
        .with_state(repo.clone())
        .layer(middleware::from_fn_with_state(
            repo.clone(),
            jwt_auth_middleware,
        ))
        .layer(middleware::from_fn_with_state(
            repo.clone(),
            maintenance_mode_middleware,
        ));

    // Admin-only routes
    // Note: jwt_auth_middleware only verifies the JWT token exists and is valid
    // The actual admin enforcement happens via the AdminClaims extractor in the handler
    // Handlers using AdminClaims(claims) will automatically reject non-admin users with 403
    let admin_routes = Router::new()
        .route("/admin/check", get(admin::check_admin_status))
        .route("/admin/users", get(admin::get_all_users))
        .route("/admin/users/:id", patch(admin::update_user_username))
        .route("/admin/users/:id", delete(admin::delete_user))
        .route("/admin/users/:id/ban", patch(admin::ban_user))
        .route("/admin/users/:id/unban", patch(admin::unban_user))
        // Character management
        .route("/admin/characters", get(admin::get_all_characters))
        .route("/admin/characters/:id", get(admin::get_character_detail))
        .route("/admin/characters/:id", patch(admin::update_character_name))
        .route(
            "/admin/characters/:id/adventures",
            patch(admin::update_character_adventures),
        )
        .route("/admin/characters/:id", delete(admin::delete_character))
        // Class management
        .route("/admin/classes", post(admin::create_class))
        .route("/admin/classes/:id", patch(admin::update_class))
        .route("/admin/classes/:id", delete(admin::delete_class))
        .route(
            "/admin/classes/:class_id/abilities",
            post(admin::add_class_ability),
        )
        .route(
            "/admin/classes/:class_id/abilities/:ability_id",
            delete(admin::remove_class_ability),
        )
        // Ability management
        .route("/admin/abilities", get(admin::get_all_abilities))
        .route("/admin/abilities", post(admin::create_ability))
        .route(
            "/admin/abilities/passive-effects",
            get(admin::get_passive_effects),
        )
        .route("/admin/abilities/:id", patch(admin::update_ability))
        .route("/admin/abilities/:id", delete(admin::delete_ability))
        // Config export/import
        .route("/admin/config/export", get(admin::export_game_config))
        .route("/admin/config/import", post(admin::import_game_config))
        // Location management
        .route("/admin/locations", get(admin::get_all_locations))
        .route("/admin/locations", post(admin::create_location))
        .route("/admin/locations/:id", patch(admin::update_location))
        .route("/admin/locations/:id", delete(admin::delete_location))
        .route(
            "/admin/locations/:location_id/creatures/:creature_id",
            post(admin::add_creature_to_location),
        )
        .route(
            "/admin/locations/:location_id/creatures/:creature_id",
            delete(admin::remove_creature_from_location),
        )
        .route(
            "/admin/locations/:location_id/adventures/:adventure_id",
            post(admin::add_adventure_to_location),
        )
        .route(
            "/admin/locations/:location_id/adventures/:adventure_id",
            delete(admin::remove_adventure_from_location),
        )
        .route(
            "/admin/locations/:location_id/creatures",
            get(admin::get_location_creatures),
        )
        .route(
            "/admin/locations/:location_id/adventures",
            get(admin::get_location_adventures),
        )
        // Creature management
        .route("/admin/creatures", get(admin::get_all_creatures))
        .route("/admin/creatures", post(admin::create_creature))
        .route("/admin/creatures/:id", get(admin::get_creature))
        .route("/admin/creatures/:id", patch(admin::update_creature))
        .route("/admin/creatures/:id", delete(admin::delete_creature))
        // Adventure management
        .route("/admin/adventures", get(admin::get_all_adventures))
        .route("/admin/adventures", post(admin::create_adventure))
        .route("/admin/adventures/:id", patch(admin::update_adventure))
        .route("/admin/adventures/:id", delete(admin::delete_adventure))
        // Maintenance mode
        .route("/admin/maintenance", get(admin::get_maintenance_mode))
        .route("/admin/maintenance", post(admin::set_maintenance_mode))
        // Manual rollover trigger
        .route("/admin/rollover", post(admin::trigger_rollover))
        .with_state(repo.clone())
        .layer(middleware::from_fn_with_state(
            repo.clone(),
            jwt_auth_middleware,
        ));

    Router::new()
        .merge(public_routes)
        .merge(chat_routes)
        .merge(protected_routes)
        .merge(admin_routes)
        .layer(middleware::from_fn(global_error_handler))
        .with_state(repo)
}
