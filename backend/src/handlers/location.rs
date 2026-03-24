use crate::game::creatures::{Creature, CreatureInCombat};
use crate::game::locations::{get_all_locations, get_location_adventures, get_location_creatures};
use crate::middleware::AuthClaims;
use crate::repository::UserRepository;
use crate::{error::AppError, game::locations::Location};
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
    pub creature: Option<CreatureInCombat>,
}

fn creature_to_combat(creature: &Creature) -> CreatureInCombat {
    CreatureInCombat {
        name: creature.name.to_string(),
        introduction_text: creature.introduction_text.to_string(),
        level: creature.level,
        health: creature.max_health,
        max_health: creature.max_health,
        might: creature.might,
        defense: creature.defense,
        magic: creature.magic,
        resistance: creature.resistance,
        agility: creature.agility,
        creature_type: Some(creature.creature_type.to_string()),
        experience_reward: creature.experience_reward,
        attack_description: creature.attack_description.to_string(),
    }
}

#[derive(Debug, Deserialize)]
pub struct VisitLocationRequest {
    #[serde(rename = "characterId")]
    pub character_id: String,
}

pub async fn get_locations() -> Result<Json<Vec<Location>>, AppError> {
    match get_all_locations() {
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
    AuthClaims(claims): AuthClaims,
    Json(request): Json<VisitLocationRequest>,
) -> Result<Json<VisitLocationResponse>, AppError> {
    // Verify character exists and belongs to the authenticated user
    let character = repo
        .get_character(&request.character_id, &claims.sub)
        .await
        .map_err(|_| AppError::character_not_found(&request.character_id))?;

    // Validate character has HP > 0
    if character.health <= 0 {
        return Err(AppError::validation_error(
            "Your character has 0 HP. You must rest before adventuring.",
        ));
    }
    // Get creatures and adventures for this location
    let creatures = get_location_creatures(&location_id)?;
    let adventures = get_location_adventures(&location_id)?;

    let mut rng = rand::thread_rng();

    // Determine if we have adventures available
    let has_adventures = !adventures.is_empty();

    if !has_adventures {
        if creatures.is_empty() {
            return Err(AppError::BadRequest(
                "No encounters available at this location".to_string(),
            ));
        }

        // Pick a random creature based on spawn rates
        let total_weight: i64 = creatures.iter().map(|(_, rate)| rate).sum();
        let mut random_value = rng.gen_range(0..total_weight);

        for (location_creature, spawn_rate) in &creatures {
            random_value -= spawn_rate;
            if random_value < 0 {
                return Ok(Json(VisitLocationResponse {
                    encounter_type: "combat".to_string(),
                    encounter_id: location_creature.id.to_string(),
                    creature: Some(creature_to_combat(location_creature)),
                }));
            }
        }

        // Fallback to first creature (shouldn't happen)
        return Ok(Json(VisitLocationResponse {
            encounter_type: "combat".to_string(),
            encounter_id: creatures[0].0.id.to_string(),
            creature: Some(creature_to_combat(&creatures[0].0)),
        }));
    }

    // We have both adventures and potentially creatures
    // 50/50 chance between combat and adventure
    let is_combat = rng.gen_bool(0.5);

    if is_combat {
        if creatures.is_empty() {
            // No creatures, force adventure instead
            let total_weight: i64 = adventures.iter().map(|(_, rate)| rate).sum();
            let mut random_value = rng.gen_range(0..total_weight);

            for (location_adventure, spawn_rate) in &adventures {
                random_value -= spawn_rate;
                if random_value < 0 {
                    return Ok(Json(VisitLocationResponse {
                        encounter_type: "adventure".to_string(),
                        encounter_id: location_adventure.adventure_id.to_string(),
                        creature: None,
                    }));
                }
            }
        }

        // Pick random creature based on spawn rates
        let total_weight: i64 = creatures.iter().map(|(_, rate)| rate).sum();
        let mut random_value = rng.gen_range(0..total_weight);

        for (location_creature, spawn_rate) in &creatures {
            random_value -= spawn_rate;
            if random_value < 0 {
                return Ok(Json(VisitLocationResponse {
                    encounter_type: "combat".to_string(),
                    encounter_id: location_creature.id.to_string(),
                    creature: Some(creature_to_combat(location_creature)),
                }));
            }
        }
    }

    // Adventure encounter
    // Pick random adventure based on spawn rates
    let total_weight: i64 = adventures.iter().map(|(_, rate)| rate).sum();
    let mut random_value = rng.gen_range(0..total_weight);

    for (location_adventure, spawn_rate) in &adventures {
        random_value -= spawn_rate;
        if random_value < 0 {
            return Ok(Json(VisitLocationResponse {
                encounter_type: "adventure".to_string(),
                encounter_id: location_adventure.adventure_id.to_string(),
                creature: None,
            }));
        }
    }

    // Fallback (shouldn't happen)
    Err(AppError::BadRequest(
        "No encounters available at this location".to_string(),
    ))
}
