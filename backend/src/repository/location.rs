use crate::models::Location;
use anyhow::{Context, Result};
use aws_sdk_dynamodb::types::AttributeValue;
use std::collections::HashMap;

use super::UserRepository;

impl UserRepository {
    // Create a new location
    pub async fn create_location(&self, location: Location) -> Result<Location> {
        let mut item = HashMap::new();
        item.insert(
            "PK".to_string(),
            AttributeValue::S(format!("LOCATION#{}", location.id)),
        );
        item.insert("SK".to_string(), AttributeValue::S("METADATA".to_string()));
        item.insert(
            "entity_type".to_string(),
            AttributeValue::S("LOCATION".to_string()),
        );
        item.insert("id".to_string(), AttributeValue::S(location.id.clone()));
        item.insert("name".to_string(), AttributeValue::S(location.name.clone()));
        item.insert(
            "description".to_string(),
            AttributeValue::S(location.description.clone()),
        );
        item.insert(
            "min_level".to_string(),
            AttributeValue::N(location.min_level.to_string()),
        );
        item.insert(
            "max_level".to_string(),
            AttributeValue::N(location.max_level.to_string()),
        );
        item.insert(
            "tier".to_string(),
            AttributeValue::N(location.tier.to_string()),
        );
        item.insert(
            "enabled".to_string(),
            AttributeValue::Bool(location.enabled),
        );
        item.insert(
            "created_at".to_string(),
            AttributeValue::S(location.created_at.clone()),
        );

        self.client
            .put_item()
            .table_name(&self.table_name)
            .set_item(Some(item))
            .send()
            .await
            .context("Failed to create location")?;

        Ok(location)
    }

    // Get a location by ID
    pub async fn get_location(&self, location_id: &str) -> Result<Location> {
        let result = self
            .client
            .get_item()
            .table_name(&self.table_name)
            .key("PK", AttributeValue::S(format!("LOCATION#{}", location_id)))
            .key("SK", AttributeValue::S("METADATA".to_string()))
            .send()
            .await
            .context("Failed to get location")?;

        match result.item() {
            Some(item) => self.parse_location(item),
            None => Err(anyhow::anyhow!("Location not found")),
        }
    }

    // Get all locations
    pub async fn get_all_locations(&self) -> Result<Vec<Location>> {
        let result = self
            .client
            .scan()
            .table_name(&self.table_name)
            .filter_expression("entity_type = :entity_type")
            .expression_attribute_values(":entity_type", AttributeValue::S("LOCATION".to_string()))
            .send()
            .await
            .context("Failed to scan locations")?;

        let mut locations = Vec::new();
        for item in result.items() {
            locations.push(self.parse_location(item)?);
        }

        Ok(locations)
    }

    // Update a location
    pub async fn update_location(&self, location_id: &str, location: Location) -> Result<Location> {
        // Delete old location and create new one
        self.delete_location(location_id).await?;
        self.create_location(location).await
    }

    // Delete a location
    pub async fn delete_location(&self, location_id: &str) -> Result<()> {
        // Delete location metadata
        self.client
            .delete_item()
            .table_name(&self.table_name)
            .key("PK", AttributeValue::S(format!("LOCATION#{}", location_id)))
            .key("SK", AttributeValue::S("METADATA".to_string()))
            .send()
            .await
            .context("Failed to delete location")?;

        // Delete all creatures for this location
        let result = self
            .client
            .query()
            .table_name(&self.table_name)
            .key_condition_expression("PK = :pk AND begins_with(SK, :sk_prefix)")
            .expression_attribute_values(
                ":pk",
                AttributeValue::S(format!("LOCATION#{}", location_id)),
            )
            .expression_attribute_values(":sk_prefix", AttributeValue::S("CREATURE#".to_string()))
            .send()
            .await
            .context("Failed to query location creatures")?;

        for item in result.items() {
            if let Some(sk) = item.get("SK").and_then(|v| v.as_s().ok()) {
                self.client
                    .delete_item()
                    .table_name(&self.table_name)
                    .key("PK", AttributeValue::S(format!("LOCATION#{}", location_id)))
                    .key("SK", AttributeValue::S(sk.to_string()))
                    .send()
                    .await
                    .context("Failed to delete location creature")?;
            }
        }

        // Delete all adventures for this location
        let result = self
            .client
            .query()
            .table_name(&self.table_name)
            .key_condition_expression("PK = :pk AND begins_with(SK, :sk_prefix)")
            .expression_attribute_values(
                ":pk",
                AttributeValue::S(format!("LOCATION#{}", location_id)),
            )
            .expression_attribute_values(":sk_prefix", AttributeValue::S("ADVENTURE#".to_string()))
            .send()
            .await
            .context("Failed to query location adventures")?;

        for item in result.items() {
            if let Some(sk) = item.get("SK").and_then(|v| v.as_s().ok()) {
                self.client
                    .delete_item()
                    .table_name(&self.table_name)
                    .key("PK", AttributeValue::S(format!("LOCATION#{}", location_id)))
                    .key("SK", AttributeValue::S(sk.to_string()))
                    .send()
                    .await
                    .context("Failed to delete location adventure")?;
            }
        }

        Ok(())
    }

    // Add creature to location
    pub async fn add_creature_to_location(
        &self,
        location_id: &str,
        creature_id: &str,
        spawn_rate: i64,
    ) -> Result<()> {
        let mut item = HashMap::new();
        item.insert(
            "PK".to_string(),
            AttributeValue::S(format!("LOCATION#{}", location_id)),
        );
        item.insert(
            "SK".to_string(),
            AttributeValue::S(format!("CREATURE#{}", creature_id)),
        );
        item.insert(
            "entity_type".to_string(),
            AttributeValue::S("LOCATION_CREATURE".to_string()),
        );
        item.insert(
            "location_id".to_string(),
            AttributeValue::S(location_id.to_string()),
        );
        item.insert(
            "creature_id".to_string(),
            AttributeValue::S(creature_id.to_string()),
        );
        item.insert(
            "spawn_rate".to_string(),
            AttributeValue::N(spawn_rate.to_string()),
        );

        self.client
            .put_item()
            .table_name(&self.table_name)
            .set_item(Some(item))
            .send()
            .await
            .context("Failed to add creature to location")?;

        Ok(())
    }

    // Remove creature from location
    pub async fn remove_creature_from_location(
        &self,
        location_id: &str,
        creature_id: &str,
    ) -> Result<()> {
        self.client
            .delete_item()
            .table_name(&self.table_name)
            .key("PK", AttributeValue::S(format!("LOCATION#{}", location_id)))
            .key("SK", AttributeValue::S(format!("CREATURE#{}", creature_id)))
            .send()
            .await
            .context("Failed to remove creature from location")?;

        Ok(())
    }

    // Add adventure to location
    pub async fn add_adventure_to_location(
        &self,
        location_id: &str,
        adventure_id: &str,
        spawn_rate: i64,
    ) -> Result<()> {
        let mut item = HashMap::new();
        item.insert(
            "PK".to_string(),
            AttributeValue::S(format!("LOCATION#{}", location_id)),
        );
        item.insert(
            "SK".to_string(),
            AttributeValue::S(format!("ADVENTURE#{}", adventure_id)),
        );
        item.insert(
            "entity_type".to_string(),
            AttributeValue::S("LOCATION_ADVENTURE".to_string()),
        );
        item.insert(
            "location_id".to_string(),
            AttributeValue::S(location_id.to_string()),
        );
        item.insert(
            "adventure_id".to_string(),
            AttributeValue::S(adventure_id.to_string()),
        );
        item.insert(
            "spawn_rate".to_string(),
            AttributeValue::N(spawn_rate.to_string()),
        );

        self.client
            .put_item()
            .table_name(&self.table_name)
            .set_item(Some(item))
            .send()
            .await
            .context("Failed to add adventure to location")?;

        Ok(())
    }

    // Remove adventure from location
    pub async fn remove_adventure_from_location(
        &self,
        location_id: &str,
        adventure_id: &str,
    ) -> Result<()> {
        self.client
            .delete_item()
            .table_name(&self.table_name)
            .key("PK", AttributeValue::S(format!("LOCATION#{}", location_id)))
            .key(
                "SK",
                AttributeValue::S(format!("ADVENTURE#{}", adventure_id)),
            )
            .send()
            .await
            .context("Failed to remove adventure from location")?;

        Ok(())
    }

    // Get creatures for a location
    pub async fn get_location_creatures(&self, location_id: &str) -> Result<Vec<(String, i64)>> {
        let result = self
            .client
            .query()
            .table_name(&self.table_name)
            .key_condition_expression("PK = :pk AND begins_with(SK, :sk_prefix)")
            .expression_attribute_values(
                ":pk",
                AttributeValue::S(format!("LOCATION#{}", location_id)),
            )
            .expression_attribute_values(":sk_prefix", AttributeValue::S("CREATURE#".to_string()))
            .send()
            .await
            .context("Failed to query location creatures")?;

        let mut creatures = Vec::new();
        for item in result.items() {
            if let (Some(creature_id), Some(spawn_rate)) = (
                item.get("creature_id").and_then(|v| v.as_s().ok()),
                item.get("spawn_rate")
                    .and_then(|v| v.as_n().ok())
                    .and_then(|s| s.parse::<i64>().ok()),
            ) {
                creatures.push((creature_id.to_string(), spawn_rate));
            }
        }

        Ok(creatures)
    }

    // Get adventures for a location
    pub async fn get_location_adventures(&self, location_id: &str) -> Result<Vec<(String, i64)>> {
        let result = self
            .client
            .query()
            .table_name(&self.table_name)
            .key_condition_expression("PK = :pk AND begins_with(SK, :sk_prefix)")
            .expression_attribute_values(
                ":pk",
                AttributeValue::S(format!("LOCATION#{}", location_id)),
            )
            .expression_attribute_values(":sk_prefix", AttributeValue::S("ADVENTURE#".to_string()))
            .send()
            .await
            .context("Failed to query location adventures")?;

        let mut adventures = Vec::new();
        for item in result.items() {
            if let (Some(adventure_id), Some(spawn_rate)) = (
                item.get("adventure_id").and_then(|v| v.as_s().ok()),
                item.get("spawn_rate")
                    .and_then(|v| v.as_n().ok())
                    .and_then(|s| s.parse::<i64>().ok()),
            ) {
                adventures.push((adventure_id.to_string(), spawn_rate));
            }
        }

        Ok(adventures)
    }

    pub(crate) fn parse_location(
        &self,
        item: &HashMap<String, AttributeValue>,
    ) -> Result<Location> {
        let get_string = |key: &str| -> Result<String> {
            item.get(key)
                .and_then(|v| v.as_s().ok())
                .map(|s| s.to_string())
                .context(format!("Missing {}", key))
        };

        let get_i64 = |key: &str| -> Result<i64> {
            item.get(key)
                .and_then(|v| v.as_n().ok())
                .and_then(|s| s.parse::<i64>().ok())
                .context(format!("Missing {}", key))
        };

        let get_bool = |key: &str| -> Result<bool> {
            item.get(key)
                .and_then(|v| v.as_bool().ok())
                .copied()
                .context(format!("Missing {}", key))
        };

        Ok(Location {
            id: get_string("id")?,
            name: get_string("name")?,
            description: get_string("description")?,
            min_level: get_i64("min_level")?,
            max_level: get_i64("max_level")?,
            tier: get_i64("tier")?,
            enabled: get_bool("enabled")?,
            created_at: get_string("created_at")?,
        })
    }
}
