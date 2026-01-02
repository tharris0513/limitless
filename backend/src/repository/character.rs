use crate::models::{Character, CharacterStats};
use anyhow::{Context, Result};
use aws_sdk_dynamodb::types::AttributeValue;
use chrono::Utc;
use std::collections::HashMap;
use uuid::Uuid;

use super::UserRepository;

impl UserRepository {
    pub async fn get_user_characters(&self, user_id: &str) -> Result<Vec<Character>> {
        let result = self
            .client
            .query()
            .table_name(&self.table_name)
            .key_condition_expression("PK = :pk AND begins_with(SK, :sk)")
            .expression_attribute_values(":pk", AttributeValue::S(format!("USER#{}", user_id)))
            .expression_attribute_values(":sk", AttributeValue::S("CHAR#".to_string()))
            .scan_index_forward(false)
            .send()
            .await
            .context("Failed to query user characters")?;

        let mut characters = Vec::new();
        for item in result.items() {
            characters.push(self.parse_character(item)?);
        }

        Ok(characters)
    }

    pub async fn create_character(
        &self,
        user_id: &str,
        name: &str,
        class_id: &str,
    ) -> Result<Character> {
        // Get class details to set initial stats
        let class = self.get_class(class_id).await?;

        let character_id = Uuid::new_v4().to_string();
        let created_at = Utc::now().to_rfc3339();
        let last_played = created_at.clone();

        let stats = CharacterStats {
            might: class.starting_might,
            defense: class.starting_defense,
            magic: class.starting_magic,
            resistance: class.starting_resistance,
            agility: class.starting_agility,
            adventures: 50,
        };

        let mut item = HashMap::new();
        item.insert(
            "PK".to_string(),
            AttributeValue::S(format!("USER#{}", user_id)),
        );
        item.insert(
            "SK".to_string(),
            AttributeValue::S(format!("CHAR#{}", character_id)),
        );
        item.insert(
            "entity_type".to_string(),
            AttributeValue::S("CHARACTER".to_string()),
        );
        item.insert("id".to_string(), AttributeValue::S(character_id.clone()));
        item.insert(
            "user_id".to_string(),
            AttributeValue::S(user_id.to_string()),
        );
        item.insert("name".to_string(), AttributeValue::S(name.to_string()));
        item.insert(
            "class_id".to_string(),
            AttributeValue::S(class_id.to_string()),
        );
        item.insert("level".to_string(), AttributeValue::N("1".to_string()));
        item.insert(
            "health".to_string(),
            AttributeValue::N(class.starting_health.to_string()),
        );
        item.insert(
            "max_health".to_string(),
            AttributeValue::N(class.starting_health.to_string()),
        );
        item.insert(
            "mana".to_string(),
            AttributeValue::N(class.starting_mana.to_string()),
        );
        item.insert(
            "max_mana".to_string(),
            AttributeValue::N(class.starting_mana.to_string()),
        );
        item.insert("experience".to_string(), AttributeValue::N("0".to_string()));
        item.insert(
            "experience_to_next".to_string(),
            AttributeValue::N("100".to_string()),
        );
        item.insert(
            "location".to_string(),
            AttributeValue::S("starting_area".to_string()),
        );
        item.insert(
            "created_at".to_string(),
            AttributeValue::S(created_at.clone()),
        );
        item.insert(
            "last_played".to_string(),
            AttributeValue::S(last_played.clone()),
        );

        // Embed stats in the same item
        item.insert(
            "might".to_string(),
            AttributeValue::N(stats.might.to_string()),
        );
        item.insert(
            "defense".to_string(),
            AttributeValue::N(stats.defense.to_string()),
        );
        item.insert(
            "magic".to_string(),
            AttributeValue::N(stats.magic.to_string()),
        );
        item.insert(
            "resistance".to_string(),
            AttributeValue::N(stats.resistance.to_string()),
        );
        item.insert(
            "agility".to_string(),
            AttributeValue::N(stats.agility.to_string()),
        );
        item.insert(
            "adventures".to_string(),
            AttributeValue::N(stats.adventures.to_string()),
        );

        self.client
            .put_item()
            .table_name(&self.table_name)
            .set_item(Some(item))
            .send()
            .await
            .context("Failed to create character")?;

        // Unlock level 1 abilities for this class
        self.unlock_character_abilities(&character_id, class_id, 1)
            .await?;

        Ok(Character {
            id: character_id,
            user_id: user_id.to_string(),
            name: name.to_string(),
            class_id: class_id.to_string(),
            level: 1,
            health: class.starting_health,
            max_health: class.starting_health,
            mana: class.starting_mana,
            max_mana: class.starting_mana,
            experience: 0,
            experience_to_next: 100,
            stats,
            location: "starting_area".to_string(),
            game_state: None,
            created_at,
            last_played,
        })
    }

    pub async fn update_character_last_played(&self, character_id: &str) -> Result<()> {
        let last_played = Utc::now().to_rfc3339();

        // We need to find the character first to get its user_id for the PK
        let result = self
            .client
            .query()
            .table_name(&self.table_name)
            .index_name("character-id-index")
            .key_condition_expression("id = :id AND entity_type = :entity_type")
            .expression_attribute_values(":id", AttributeValue::S(character_id.to_string()))
            .expression_attribute_values(":entity_type", AttributeValue::S("CHARACTER".to_string()))
            .send()
            .await
            .context("Failed to find character for update")?;

        if let Some(item) = result.items().first() {
            let user_id = item
                .get("user_id")
                .and_then(|v| v.as_s().ok())
                .context("Missing user_id")?;

            self.client
                .update_item()
                .table_name(&self.table_name)
                .key("PK", AttributeValue::S(format!("USER#{}", user_id)))
                .key("SK", AttributeValue::S(format!("CHAR#{}", character_id)))
                .update_expression("SET last_played = :last_played")
                .expression_attribute_values(":last_played", AttributeValue::S(last_played))
                .send()
                .await
                .context("Failed to update character last_played")?;
        }

        Ok(())
    }

    pub async fn update_character_game_state(
        &self,
        character_id: &str,
        user_id: &str,
        game_state: Option<String>,
    ) -> Result<()> {
        let update_expression = if game_state.is_some() {
            "SET game_state = :game_state"
        } else {
            "REMOVE game_state"
        };

        let mut request = self
            .client
            .update_item()
            .table_name(&self.table_name)
            .key("PK", AttributeValue::S(format!("USER#{}", user_id)))
            .key("SK", AttributeValue::S(format!("CHAR#{}", character_id)))
            .update_expression(update_expression);

        if let Some(state) = game_state {
            request = request.expression_attribute_values(":game_state", AttributeValue::S(state));
        }

        request
            .send()
            .await
            .context("Failed to update character game_state")?;

        Ok(())
    }

    pub(crate) fn parse_character(
        &self,
        item: &HashMap<String, AttributeValue>,
    ) -> Result<Character> {
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

        Ok(Character {
            id: get_string("id")?,
            user_id: get_string("user_id")?,
            name: get_string("name")?,
            class_id: get_string("class_id")?,
            level: get_i64("level")?,
            health: get_i64("health")?,
            max_health: get_i64("max_health")?,
            mana: get_i64("mana")?,
            max_mana: get_i64("max_mana")?,
            experience: get_i64("experience")?,
            experience_to_next: get_i64("experience_to_next")?,
            stats: CharacterStats {
                might: get_i64("might")?,
                defense: get_i64("defense")?,
                magic: get_i64("magic")?,
                resistance: get_i64("resistance")?,
                agility: get_i64("agility")?,
                adventures: get_i64("adventures")?,
            },
            location: get_string("location")?,
            game_state: item
                .get("game_state")
                .and_then(|v| v.as_s().ok())
                .map(|s| s.to_string()),
            created_at: get_string("created_at")?,
            last_played: get_string("last_played")?,
        })
    }

    // Admin only - get all characters from all users
    pub async fn get_all_characters(&self) -> Result<Vec<Character>> {
        let result = self
            .client
            .scan()
            .table_name(&self.table_name)
            .filter_expression("entity_type = :entity_type")
            .expression_attribute_values(":entity_type", AttributeValue::S("CHARACTER".to_string()))
            .send()
            .await
            .context("Failed to scan all characters")?;

        let mut characters = Vec::new();
        for item in result.items() {
            characters.push(self.parse_character(item)?);
        }

        Ok(characters)
    }

    // Admin only - update character name
    pub async fn update_character_name(&self, character_id: &str, name: &str) -> Result<Character> {
        // Find the character first to get its user_id for the PK
        let result = self
            .client
            .scan()
            .table_name(&self.table_name)
            .filter_expression("id = :id AND entity_type = :entity_type")
            .expression_attribute_values(":id", AttributeValue::S(character_id.to_string()))
            .expression_attribute_values(":entity_type", AttributeValue::S("CHARACTER".to_string()))
            .send()
            .await
            .context("Failed to find character for update")?;

        if let Some(item) = result.items().first() {
            let user_id = item
                .get("user_id")
                .and_then(|v| v.as_s().ok())
                .context("Missing user_id")?;

            self.client
                .update_item()
                .table_name(&self.table_name)
                .key("PK", AttributeValue::S(format!("USER#{}", user_id)))
                .key("SK", AttributeValue::S(format!("CHAR#{}", character_id)))
                .update_expression("SET #name = :name")
                .expression_attribute_names("#name", "name")
                .expression_attribute_values(":name", AttributeValue::S(name.to_string()))
                .send()
                .await
                .context("Failed to update character name")?;

            // Fetch and return updated character
            let updated_result = self
                .client
                .get_item()
                .table_name(&self.table_name)
                .key("PK", AttributeValue::S(format!("USER#{}", user_id)))
                .key("SK", AttributeValue::S(format!("CHAR#{}", character_id)))
                .send()
                .await
                .context("Failed to fetch updated character")?;

            if let Some(updated_item) = updated_result.item() {
                return self.parse_character(updated_item);
            }
        }

        Err(anyhow::anyhow!("Character not found"))
    }

    // Admin only - update character adventures
    pub async fn update_character_adventures(
        &self,
        character_id: &str,
        adventures: i64,
    ) -> Result<Character> {
        // Find the character first to get its user_id for the PK
        let result = self
            .client
            .scan()
            .table_name(&self.table_name)
            .filter_expression("id = :id AND entity_type = :entity_type")
            .expression_attribute_values(":id", AttributeValue::S(character_id.to_string()))
            .expression_attribute_values(":entity_type", AttributeValue::S("CHARACTER".to_string()))
            .send()
            .await
            .context("Failed to find character for update")?;

        if let Some(item) = result.items().first() {
            let user_id = item
                .get("user_id")
                .and_then(|v| v.as_s().ok())
                .context("Missing user_id")?;

            self.client
                .update_item()
                .table_name(&self.table_name)
                .key("PK", AttributeValue::S(format!("USER#{}", user_id)))
                .key("SK", AttributeValue::S(format!("CHAR#{}", character_id)))
                .update_expression("SET adventures = :adventures")
                .expression_attribute_values(
                    ":adventures",
                    AttributeValue::N(adventures.to_string()),
                )
                .send()
                .await
                .context("Failed to update character adventures")?;

            // Fetch and return updated character
            let updated_result = self
                .client
                .get_item()
                .table_name(&self.table_name)
                .key("PK", AttributeValue::S(format!("USER#{}", user_id)))
                .key("SK", AttributeValue::S(format!("CHAR#{}", character_id)))
                .send()
                .await
                .context("Failed to fetch updated character")?;

            if let Some(updated_item) = updated_result.item() {
                return self.parse_character(updated_item);
            }
        }

        Err(anyhow::anyhow!("Character not found"))
    }

    // Admin only - delete character
    pub async fn delete_character(&self, character_id: &str) -> Result<()> {
        // Find the character first to get its user_id for the PK
        let result = self
            .client
            .scan()
            .table_name(&self.table_name)
            .filter_expression("id = :id AND entity_type = :entity_type")
            .expression_attribute_values(":id", AttributeValue::S(character_id.to_string()))
            .expression_attribute_values(":entity_type", AttributeValue::S("CHARACTER".to_string()))
            .send()
            .await
            .context("Failed to find character for deletion")?;

        if let Some(item) = result.items().first() {
            let user_id = item
                .get("user_id")
                .and_then(|v| v.as_s().ok())
                .context("Missing user_id")?;

            self.client
                .delete_item()
                .table_name(&self.table_name)
                .key("PK", AttributeValue::S(format!("USER#{}", user_id)))
                .key("SK", AttributeValue::S(format!("CHAR#{}", character_id)))
                .send()
                .await
                .context("Failed to delete character")?;

            tracing::info!("Deleted character {}", character_id);
            return Ok(());
        }

        Err(anyhow::anyhow!("Character not found"))
    }
}
