use crate::level_system::{calculate_level_from_experience, calculate_stat_increases_for_level};
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

    /// Grant experience to a character and handle level-ups automatically.
    /// Returns the updated character and a list of levels gained (for notification purposes).
    pub async fn grant_experience(
        &self,
        character_id: &str,
        experience_gain: i64,
    ) -> Result<(Character, Vec<i64>)> {
        // Find the character first
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
            .context("Failed to find character")?;

        if let Some(item) = result.items().first() {
            let character = self.parse_character(item)?;
            let user_id = &character.user_id;

            // Calculate new total experience
            let new_total_experience = character.experience + experience_gain;

            // Determine new level and experience progress
            let (new_level, exp_into_level, exp_for_next_level) =
                calculate_level_from_experience(new_total_experience);

            let old_level = character.level;
            let levels_gained: Vec<i64> = ((old_level + 1)..=new_level).collect();

            // Calculate stat increases for all levels gained
            let mut total_might = 0;
            let mut total_defense = 0;
            let mut total_magic = 0;
            let mut total_resistance = 0;
            let mut total_agility = 0;
            let mut total_max_health = 0;
            let mut total_max_mana = 0;

            for level in &levels_gained {
                let (m, d, mag, r, a, hp, mp) =
                    calculate_stat_increases_for_level(&character.class_id, *level);
                total_might += m;
                total_defense += d;
                total_magic += mag;
                total_resistance += r;
                total_agility += a;
                total_max_health += hp;
                total_max_mana += mp;
            }

            // Build update expression
            let mut update_parts = vec![
                "experience = :experience".to_string(),
                "experience_to_next = :experience_to_next".to_string(),
            ];

            let mut attr_values = HashMap::new();
            attr_values.insert(
                ":experience".to_string(),
                AttributeValue::N(exp_into_level.to_string()),
            );
            attr_values.insert(
                ":experience_to_next".to_string(),
                AttributeValue::N(exp_for_next_level.to_string()),
            );

            // If leveled up, update level and stats
            if new_level > old_level {
                update_parts.push("level = :level".to_string());
                attr_values.insert(
                    ":level".to_string(),
                    AttributeValue::N(new_level.to_string()),
                );

                // Update stats
                let new_might = character.stats.might + total_might;
                let new_defense = character.stats.defense + total_defense;
                let new_magic = character.stats.magic + total_magic;
                let new_resistance = character.stats.resistance + total_resistance;
                let new_agility = character.stats.agility + total_agility;
                let new_max_health = character.max_health + total_max_health;
                let new_max_mana = character.max_mana + total_max_mana;

                // Also increase current health and mana by the same amount (heal on level up)
                let new_health = character.health + total_max_health;
                let new_mana = character.mana + total_max_mana;

                update_parts.extend([
                    "might = :might".to_string(),
                    "defense = :defense".to_string(),
                    "magic = :magic".to_string(),
                    "resistance = :resistance".to_string(),
                    "agility = :agility".to_string(),
                    "max_health = :max_health".to_string(),
                    "max_mana = :max_mana".to_string(),
                    "health = :health".to_string(),
                    "mana = :mana".to_string(),
                ]);

                attr_values.insert(
                    ":might".to_string(),
                    AttributeValue::N(new_might.to_string()),
                );
                attr_values.insert(
                    ":defense".to_string(),
                    AttributeValue::N(new_defense.to_string()),
                );
                attr_values.insert(
                    ":magic".to_string(),
                    AttributeValue::N(new_magic.to_string()),
                );
                attr_values.insert(
                    ":resistance".to_string(),
                    AttributeValue::N(new_resistance.to_string()),
                );
                attr_values.insert(
                    ":agility".to_string(),
                    AttributeValue::N(new_agility.to_string()),
                );
                attr_values.insert(
                    ":max_health".to_string(),
                    AttributeValue::N(new_max_health.to_string()),
                );
                attr_values.insert(
                    ":max_mana".to_string(),
                    AttributeValue::N(new_max_mana.to_string()),
                );
                attr_values.insert(
                    ":health".to_string(),
                    AttributeValue::N(new_health.to_string()),
                );
                attr_values.insert(":mana".to_string(), AttributeValue::N(new_mana.to_string()));

                // Unlock abilities for newly gained levels
                for level in &levels_gained {
                    self.unlock_character_abilities(character_id, &character.class_id, *level)
                        .await?;
                }
            }

            let update_expression = format!("SET {}", update_parts.join(", "));

            // Perform the update
            self.client
                .update_item()
                .table_name(&self.table_name)
                .key("PK", AttributeValue::S(format!("USER#{}", user_id)))
                .key("SK", AttributeValue::S(format!("CHAR#{}", character_id)))
                .update_expression(update_expression)
                .set_expression_attribute_values(Some(attr_values))
                .send()
                .await
                .context("Failed to update character with experience")?;

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
                let updated_character = self.parse_character(updated_item)?;
                tracing::info!(
                    "Granted {} XP to character {}. Level {} -> {}",
                    experience_gain,
                    character_id,
                    old_level,
                    new_level
                );
                return Ok((updated_character, levels_gained));
            }
        }

        Err(anyhow::anyhow!("Character not found"))
    }

    // Get all characters of a specific class
    pub async fn get_characters_by_class(&self, class_id: &str) -> Result<Vec<Character>> {
        let result = self
            .client
            .scan()
            .table_name(&self.table_name)
            .filter_expression("entity_type = :entity_type AND class_id = :class_id")
            .expression_attribute_values(":entity_type", AttributeValue::S("CHARACTER".to_string()))
            .expression_attribute_values(":class_id", AttributeValue::S(class_id.to_string()))
            .send()
            .await
            .context("Failed to scan characters by class")?;

        let mut characters = Vec::new();
        for item in result.items() {
            characters.push(self.parse_character(item)?);
        }

        Ok(characters)
    }

    // Sync character abilities based on their class and level
    // This grants/removes abilities to match what the character should have at their current level
    pub async fn sync_character_abilities(&self, character_id: &str) -> Result<()> {
        // Get the character to know their class and level
        let result = self
            .client
            .scan()
            .table_name(&self.table_name)
            .filter_expression("entity_type = :entity_type AND id = :character_id")
            .expression_attribute_values(":entity_type", AttributeValue::S("CHARACTER".to_string()))
            .expression_attribute_values(
                ":character_id",
                AttributeValue::S(character_id.to_string()),
            )
            .send()
            .await
            .context("Failed to find character")?;

        let character = result
            .items()
            .first()
            .map(|item| self.parse_character(item))
            .transpose()?
            .ok_or_else(|| anyhow::anyhow!("Character not found"))?;

        // Get all class abilities
        let class_abilities = self.get_class_abilities(&character.class_id).await?;

        // Get current character abilities
        let current_abilities_result = self
            .client
            .query()
            .table_name(&self.table_name)
            .key_condition_expression("PK = :pk AND begins_with(SK, :sk)")
            .expression_attribute_values(":pk", AttributeValue::S(format!("CHAR#{}", character_id)))
            .expression_attribute_values(":sk", AttributeValue::S("ABILITY#".to_string()))
            .send()
            .await
            .context("Failed to query character abilities")?;

        let current_ability_ids: std::collections::HashSet<String> = current_abilities_result
            .items()
            .iter()
            .filter_map(|item| {
                item.get("ability_id")
                    .and_then(|v| v.as_s().ok())
                    .map(|s| s.to_string())
            })
            .collect();

        // Determine which abilities the character should have based on their level
        for (ability, unlock_level) in &class_abilities {
            if character.level >= *unlock_level {
                // Character should have this ability
                if !current_ability_ids.contains(&ability.id) {
                    // Grant the ability
                    let mut item = HashMap::new();
                    item.insert(
                        "PK".to_string(),
                        AttributeValue::S(format!("CHAR#{}", character_id)),
                    );
                    item.insert(
                        "SK".to_string(),
                        AttributeValue::S(format!("ABILITY#{}", ability.id)),
                    );
                    item.insert(
                        "entity_type".to_string(),
                        AttributeValue::S("CHARACTER_ABILITY".to_string()),
                    );
                    item.insert(
                        "character_id".to_string(),
                        AttributeValue::S(character_id.to_string()),
                    );
                    item.insert(
                        "ability_id".to_string(),
                        AttributeValue::S(ability.id.clone()),
                    );
                    item.insert(
                        "unlocked_at_level".to_string(),
                        AttributeValue::N(character.level.to_string()),
                    );

                    self.client
                        .put_item()
                        .table_name(&self.table_name)
                        .set_item(Some(item))
                        .send()
                        .await
                        .context("Failed to grant ability to character")?;

                    tracing::info!(
                        "Granted ability {} to character {} (level {})",
                        ability.id,
                        character_id,
                        character.level
                    );
                }
            } else {
                // Character should NOT have this ability (level too low)
                if current_ability_ids.contains(&ability.id) {
                    // Remove the ability
                    self.client
                        .delete_item()
                        .table_name(&self.table_name)
                        .key("PK", AttributeValue::S(format!("CHAR#{}", character_id)))
                        .key("SK", AttributeValue::S(format!("ABILITY#{}", ability.id)))
                        .send()
                        .await
                        .context("Failed to remove ability from character")?;

                    tracing::info!(
                        "Removed ability {} from character {} (level too low)",
                        ability.id,
                        character_id
                    );
                }
            }
        }

        // Remove abilities that are no longer in the class ability list
        for current_ability_id in current_ability_ids {
            let still_in_class = class_abilities
                .iter()
                .any(|(ability, _)| ability.id == current_ability_id);

            if !still_in_class {
                self.client
                    .delete_item()
                    .table_name(&self.table_name)
                    .key("PK", AttributeValue::S(format!("CHAR#{}", character_id)))
                    .key(
                        "SK",
                        AttributeValue::S(format!("ABILITY#{}", current_ability_id)),
                    )
                    .send()
                    .await
                    .context("Failed to remove obsolete ability from character")?;

                tracing::info!(
                    "Removed obsolete ability {} from character {}",
                    current_ability_id,
                    character_id
                );
            }
        }

        Ok(())
    }

    // Sync abilities for all characters of a specific class
    pub async fn sync_class_abilities(&self, class_id: &str) -> Result<usize> {
        let characters = self.get_characters_by_class(class_id).await?;
        let count = characters.len();

        for character in characters {
            if let Err(e) = self.sync_character_abilities(&character.id).await {
                tracing::error!(
                    "Failed to sync abilities for character {}: {:?}",
                    character.id,
                    e
                );
            }
        }

        tracing::info!(
            "Synced abilities for {} characters of class {}",
            count,
            class_id
        );

        Ok(count)
    }
}
