use axum::Json;
use serde_json::json;

pub async fn get_player() -> Json<serde_json::Value> {
    // Simple mock player data
    Json(json!({
        "id": "player_1",
        "username": "demo",
        "level": 1,
        "might": 10,
        "defense": 10,
        "magic": 10,
        "resistance": 10,
        "agility": 10
    }))
}
