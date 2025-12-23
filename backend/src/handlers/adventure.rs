use axum::{extract::Path, Json};
use serde_json::json;

pub async fn start_adventure(Path(adventure_id): Path<String>) -> Json<serde_json::Value> {
    // Simple mock adventure response
    Json(json!({
        "adventure_id": adventure_id,
        "status": "completed",
        "message": "Adventure completed successfully!"
    }))
}
