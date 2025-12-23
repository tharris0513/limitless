use axum::Json;
use serde_json::json;

pub async fn get_locations() -> Json<serde_json::Value> {
    // Simple mock location data
    Json(json!([
        {
            "id": "hometown",
            "name": "Hometown",
            "description": "Your starting location"
        }
    ]))
}
