use crate::error::AppError;
use crate::models::Location;
use crate::repository::UserRepository;
use axum::{extract::State, Json};
use std::sync::Arc;

pub async fn get_locations(
    State(repo): State<Arc<UserRepository>>,
) -> Result<Json<Vec<Location>>, AppError> {
    match repo.get_all_locations().await {
        Ok(locations) => {
            // Filter out disabled locations
            let enabled_locations: Vec<Location> =
                locations.into_iter().filter(|loc| loc.enabled).collect();
            Ok(Json(enabled_locations))
        }
        Err(e) => {
            tracing::error!("Failed to get locations: {:?}", e);
            Err(AppError::from(e))
        }
    }
}
