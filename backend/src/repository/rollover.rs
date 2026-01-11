use anyhow::{Context, Result};
use aws_sdk_dynamodb::types::AttributeValue;
use std::collections::HashMap;

use super::UserRepository;

impl UserRepository {
    /// Get JWT generation from database
    pub async fn get_jwt_generation(&self) -> Result<usize> {
        let result = self
            .client
            .get_item()
            .table_name(&self.table_name)
            .key("PK", AttributeValue::S("CONFIG#JWT".to_string()))
            .key("SK", AttributeValue::S("GENERATION".to_string()))
            .send()
            .await;

        match result {
            Ok(output) => {
                if let Some(item) = output.item() {
                    let generation = item
                        .get("generation")
                        .and_then(|v| v.as_n().ok())
                        .and_then(|n| n.parse::<usize>().ok())
                        .unwrap_or(0);
                    Ok(generation)
                } else {
                    Ok(0)
                }
            }
            Err(_) => Ok(0),
        }
    }

    /// Set JWT generation in database
    pub async fn set_jwt_generation(&self, generation: usize) -> Result<()> {
        let mut item = HashMap::new();
        item.insert(
            "PK".to_string(),
            AttributeValue::S("CONFIG#JWT".to_string()),
        );
        item.insert(
            "SK".to_string(),
            AttributeValue::S("GENERATION".to_string()),
        );
        item.insert(
            "entity_type".to_string(),
            AttributeValue::S("CONFIG".to_string()),
        );
        item.insert(
            "generation".to_string(),
            AttributeValue::N(generation.to_string()),
        );
        item.insert(
            "updated_at".to_string(),
            AttributeValue::S(chrono::Utc::now().to_rfc3339()),
        );

        self.client
            .put_item()
            .table_name(&self.table_name)
            .set_item(Some(item))
            .send()
            .await
            .context("Failed to set JWT generation")?;

        Ok(())
    }

    /// Increment JWT generation (invalidates all existing tokens)
    pub async fn increment_jwt_generation(&self) -> Result<usize> {
        let current = self.get_jwt_generation().await?;
        let new_gen = current + 1;
        self.set_jwt_generation(new_gen).await?;
        Ok(new_gen)
    }

    /// Get all characters across all users for rollover
    pub async fn get_all_characters_for_rollover(&self) -> Result<Vec<(String, String)>> {
        // Returns Vec<(user_id, character_id)>
        let result = self
            .client
            .scan()
            .table_name(&self.table_name)
            .filter_expression("entity_type = :entity_type")
            .expression_attribute_values(":entity_type", AttributeValue::S("CHARACTER".to_string()))
            .send()
            .await
            .context("Failed to scan characters")?;

        let mut characters = Vec::new();
        for item in result.items() {
            let user_id = item
                .get("user_id")
                .and_then(|v| v.as_s().ok())
                .map(|s| s.to_string());
            let character_id = item
                .get("id")
                .and_then(|v| v.as_s().ok())
                .map(|s| s.to_string());

            if let (Some(uid), Some(cid)) = (user_id, character_id) {
                characters.push((uid, cid));
            }
        }

        Ok(characters)
    }

    /// Add adventures to a character with max cap and restore health/mana
    pub async fn add_adventures_to_character(
        &self,
        user_id: &str,
        character_id: &str,
        amount: i32,
        max_adventures: i32,
    ) -> Result<()> {
        // First get current adventures and max stats
        let character = self.get_character(character_id, user_id).await?;
        let current_adventures = character.stats.adventures;
        let new_adventures = (current_adventures as i32 + amount).min(max_adventures);
        let max_health = character.stats.max_health;
        let max_mana = character.stats.max_mana;

        // Update adventures, health, and mana (stored as top-level attributes)
        self.client
            .update_item()
            .table_name(&self.table_name)
            .key("PK", AttributeValue::S(format!("USER#{}", user_id)))
            .key("SK", AttributeValue::S(format!("CHAR#{}", character_id)))
            .update_expression("SET adventures = :adventures, health = :health, mana = :mana")
            .expression_attribute_values(
                ":adventures",
                AttributeValue::N(new_adventures.to_string()),
            )
            .expression_attribute_values(":health", AttributeValue::N(max_health.to_string()))
            .expression_attribute_values(":mana", AttributeValue::N(max_mana.to_string()))
            .send()
            .await
            .context("Failed to update character adventures, health, and mana")?;

        Ok(())
    }
}
