use crate::error::AppError;
use crate::models::Location;
use crate::repository::UserRepository;
use axum::{
    extract::{Path, State},
    Json,
};
use rand::Rng;
use serde::{Deserialize, Serialize};
use std::sync::Arc;

#[derive(Debug, Serialize, Deserialize)]
pub struct VisitLocationResponse {
    #[serde(rename = "encounterType")]
    pub encounter_type: String, // "combat" or "adventure"
    #[serde(rename = "encounterId")]
    pub encounter_id: String, // creature_id or adventure_id
}

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

pub async fn visit_location(
    State(repo): State<Arc<UserRepository>>,
    Path(location_id): Path<String>,
) -> Result<Json<VisitLocationResponse>, AppError> {
    // Get creatures and adventures for this location
    let creatures_result = repo.get_location_creatures(&location_id).await;
    let adventures_result = repo.get_location_adventures(&location_id).await;

    let mut rng = rand::thread_rng();

    // Determine if we have adventures available
    let has_adventures = match &adventures_result {
        Ok(adventures) => !adventures.is_empty(),
        Err(_) => false,
    };

    if !has_adventures {
        // No adventures - always combat
        let creatures = creatures_result.map_err(|e| {
            tracing::error!("Failed to get location creatures: {:?}", e);
            AppError::from(e)
        })?;

        if creatures.is_empty() {
            return Err(AppError::BadRequest(
                "No encounters available at this location".to_string(),
            ));
        }

        // Pick a random creature based on spawn rates
        let total_weight: i64 = creatures.iter().map(|(_, rate)| rate).sum();
        let mut random_value = rng.gen_range(0..total_weight);

        for (creature_id, spawn_rate) in &creatures {
            random_value -= spawn_rate;
            if random_value < 0 {
                return Ok(Json(VisitLocationResponse {
                    encounter_type: "combat".to_string(),
                    encounter_id: creature_id.clone(),
                }));
            }
        }

        // Fallback to first creature (shouldn't happen)
        return Ok(Json(VisitLocationResponse {
            encounter_type: "combat".to_string(),
            encounter_id: creatures[0].0.clone(),
        }));
    }

    // We have both adventures and potentially creatures
    // 50/50 chance between combat and adventure
    let is_combat = rng.gen_bool(0.5);

    // Extract adventures once to avoid moving twice
    let adventures = match adventures_result {
        Ok(adventures) => adventures,
        Err(e) => {
            tracing::error!("Failed to get location adventures: {:?}", e);
            return Err(AppError::from(e));
        }
    };

    if is_combat {
        let creatures = creatures_result.map_err(|e| {
            tracing::error!("Failed to get location creatures: {:?}", e);
            AppError::from(e)
        })?;

        if creatures.is_empty() {
            // No creatures, force adventure instead
            let total_weight: i64 = adventures.iter().map(|(_, rate)| rate).sum();
            let mut random_value = rng.gen_range(0..total_weight);

            for (adventure_id, spawn_rate) in &adventures {
                random_value -= spawn_rate;
                if random_value < 0 {
                    return Ok(Json(VisitLocationResponse {
                        encounter_type: "adventure".to_string(),
                        encounter_id: adventure_id.clone(),
                    }));
                }
            }
        }

        // Pick random creature based on spawn rates
        let total_weight: i64 = creatures.iter().map(|(_, rate)| rate).sum();
        let mut random_value = rng.gen_range(0..total_weight);

        for (creature_id, spawn_rate) in &creatures {
            random_value -= spawn_rate;
            if random_value < 0 {
                return Ok(Json(VisitLocationResponse {
                    encounter_type: "combat".to_string(),
                    encounter_id: creature_id.clone(),
                }));
            }
        }
    }

    // Adventure encounter
    // Pick random adventure based on spawn rates
    let total_weight: i64 = adventures.iter().map(|(_, rate)| rate).sum();
    let mut random_value = rng.gen_range(0..total_weight);

    for (adventure_id, spawn_rate) in adventures {
        random_value -= spawn_rate;
        if random_value < 0 {
            return Ok(Json(VisitLocationResponse {
                encounter_type: "adventure".to_string(),
                encounter_id: adventure_id,
            }));
        }
    }

    // Fallback (shouldn't happen)
    Err(AppError::BadRequest(
        "No encounters available at this location".to_string(),
    ))
}
