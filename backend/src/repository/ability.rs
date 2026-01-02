use crate::models::{Ability, CharacterAbility};
use anyhow::{Context, Result};
use aws_sdk_dynamodb::types::AttributeValue;
use std::collections::HashMap;

use super::UserRepository;

impl UserRepository {
    // Create a new ability
    pub async fn create_ability(&self, ability: Ability) -> Result<Ability> {
        let mut item = HashMap::new();
        item.insert(
            "PK".to_string(),
            AttributeValue::S(format!("ABILITY#{}", ability.id)),
        );
        item.insert("SK".to_string(), AttributeValue::S("METADATA".to_string()));
        item.insert(
            "entity_type".to_string(),
            AttributeValue::S("ABILITY".to_string()),
        );
        item.insert("id".to_string(), AttributeValue::S(ability.id.clone()));
        item.insert("name".to_string(), AttributeValue::S(ability.name.clone()));
        item.insert(
            "description".to_string(),
            AttributeValue::S(ability.description.clone()),
        );
        item.insert(
            "ability_type".to_string(),
            AttributeValue::S(ability.ability_type.clone()),
        );
        item.insert(
            "mana_cost".to_string(),
            AttributeValue::N(ability.mana_cost.to_string()),
        );
        item.insert(
            "cooldown".to_string(),
            AttributeValue::N(ability.cooldown.to_string()),
        );

        if let Some(damage_formula) = &ability.damage_formula {
            item.insert(
                "damage_formula".to_string(),
                AttributeValue::S(damage_formula.clone()),
            );
        }
        if let Some(heal_formula) = &ability.heal_formula {
            item.insert(
                "heal_formula".to_string(),
                AttributeValue::S(heal_formula.clone()),
            );
        }
        if let Some(effect_formula) = &ability.effect_formula {
            item.insert(
                "effect_formula".to_string(),
                AttributeValue::S(effect_formula.clone()),
            );
        }

        self.client
            .put_item()
            .table_name(&self.table_name)
            .set_item(Some(item))
            .send()
            .await
            .context("Failed to create ability")?;

        Ok(ability)
    }

    // Get a single ability by ID
    pub async fn get_ability(&self, ability_id: &str) -> Result<Ability> {
        let result = self
            .client
            .get_item()
            .table_name(&self.table_name)
            .key("PK", AttributeValue::S(format!("ABILITY#{}", ability_id)))
            .key("SK", AttributeValue::S("METADATA".to_string()))
            .send()
            .await
            .context("Failed to get ability")?;

        match result.item() {
            Some(item) => self.parse_ability(item),
            None => Err(anyhow::anyhow!("Ability not found")),
        }
    }

    // Get all abilities
    pub async fn get_all_abilities(&self) -> Result<Vec<Ability>> {
        let result = self
            .client
            .scan()
            .table_name(&self.table_name)
            .filter_expression("entity_type = :entity_type")
            .expression_attribute_values(":entity_type", AttributeValue::S("ABILITY".to_string()))
            .send()
            .await
            .context("Failed to scan abilities")?;

        let mut abilities = Vec::new();
        for item in result.items() {
            abilities.push(self.parse_ability(item)?);
        }

        Ok(abilities)
    }

    // Update an ability
    pub async fn update_ability(&self, ability_id: &str, ability: Ability) -> Result<Ability> {
        // Delete old ability and create new one
        self.delete_ability(ability_id).await?;
        self.create_ability(ability).await
    }

    // Delete an ability
    pub async fn delete_ability(&self, ability_id: &str) -> Result<()> {
        self.client
            .delete_item()
            .table_name(&self.table_name)
            .key("PK", AttributeValue::S(format!("ABILITY#{}", ability_id)))
            .key("SK", AttributeValue::S("METADATA".to_string()))
            .send()
            .await
            .context("Failed to delete ability")?;

        Ok(())
    }

    pub(crate) fn parse_ability(&self, item: &HashMap<String, AttributeValue>) -> Result<Ability> {
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

        Ok(Ability {
            id: get_string("id")?,
            name: get_string("name")?,
            description: get_string("description")?,
            ability_type: get_string("ability_type")?,
            mana_cost: get_i64("mana_cost")?,
            cooldown: get_i64("cooldown")?,
            damage_formula: item
                .get("damage_formula")
                .and_then(|v| v.as_s().ok())
                .map(|s| s.to_string()),
            heal_formula: item
                .get("heal_formula")
                .and_then(|v| v.as_s().ok())
                .map(|s| s.to_string()),
            effect_formula: item
                .get("effect_formula")
                .and_then(|v| v.as_s().ok())
                .map(|s| s.to_string()),
        })
    }

    // Get abilities for a class at a specific level
    pub async fn get_class_abilities(&self, class_id: &str) -> Result<Vec<(Ability, i64)>> {
        let result = self
            .client
            .query()
            .table_name(&self.table_name)
            .key_condition_expression("PK = :pk AND begins_with(SK, :sk_prefix)")
            .expression_attribute_values(":pk", AttributeValue::S(format!("CLASS#{}", class_id)))
            .expression_attribute_values(":sk_prefix", AttributeValue::S("ABILITY#".to_string()))
            .send()
            .await
            .context("Failed to query class abilities")?;

        let mut abilities = Vec::new();
        for item in result.items() {
            if let (Some(ability), Some(level)) = (
                self.parse_ability_from_class_item(item).ok(),
                self.get_unlock_level(item).ok(),
            ) {
                abilities.push((ability, level));
            }
        }

        Ok(abilities)
    }

    // Get character's unlocked abilities
    pub async fn get_character_abilities(
        &self,
        character_id: &str,
    ) -> Result<Vec<CharacterAbility>> {
        let result = self
            .client
            .query()
            .table_name(&self.table_name)
            .key_condition_expression("PK = :pk AND begins_with(SK, :sk_prefix)")
            .expression_attribute_values(":pk", AttributeValue::S(format!("CHAR#{}", character_id)))
            .expression_attribute_values(":sk_prefix", AttributeValue::S("ABILITY#".to_string()))
            .send()
            .await
            .context("Failed to query character abilities")?;

        let mut abilities = Vec::new();
        for item in result.items() {
            abilities.push(self.parse_character_ability(item)?);
        }

        Ok(abilities)
    }

    // Unlock abilities for a character at a specific level
    pub(crate) async fn unlock_character_abilities(
        &self,
        character_id: &str,
        class_id: &str,
        level: i64,
    ) -> Result<()> {
        let class_abilities = self.get_class_abilities(class_id).await?;

        for (ability, unlock_level) in class_abilities {
            if unlock_level == level {
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
                    AttributeValue::N(level.to_string()),
                );

                self.client
                    .put_item()
                    .table_name(&self.table_name)
                    .set_item(Some(item))
                    .send()
                    .await
                    .context("Failed to unlock ability")?;
            }
        }

        Ok(())
    }

    pub(crate) fn parse_ability_from_class_item(
        &self,
        item: &HashMap<String, AttributeValue>,
    ) -> Result<Ability> {
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

        Ok(Ability {
            id: get_string("ability_id")?,
            name: get_string("ability_name")?,
            description: get_string("ability_description")?,
            ability_type: get_string("ability_type")?,
            mana_cost: get_i64("mana_cost")?,
            cooldown: get_i64("cooldown")?,
            damage_formula: item
                .get("damage_formula")
                .and_then(|v| v.as_s().ok())
                .map(|s| s.to_string()),
            heal_formula: item
                .get("heal_formula")
                .and_then(|v| v.as_s().ok())
                .map(|s| s.to_string()),
            effect_formula: item
                .get("effect_formula")
                .and_then(|v| v.as_s().ok())
                .map(|s| s.to_string()),
        })
    }

    pub(crate) fn get_unlock_level(&self, item: &HashMap<String, AttributeValue>) -> Result<i64> {
        item.get("unlock_level")
            .and_then(|v| v.as_n().ok())
            .and_then(|s| s.parse::<i64>().ok())
            .context("Missing unlock_level")
    }

    pub(crate) fn parse_character_ability(
        &self,
        item: &HashMap<String, AttributeValue>,
    ) -> Result<CharacterAbility> {
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

        Ok(CharacterAbility {
            character_id: get_string("character_id")?,
            ability_id: get_string("ability_id")?,
            unlocked_at_level: get_i64("unlocked_at_level")?,
            ability: None, // Will be populated by join if needed
        })
    }

    // Get class abilities with their full data (helper for export)
    pub(crate) async fn get_class_abilities_full(&self, class_id: &str) -> Result<Vec<(String, i64)>> {
        let result = self
            .client
            .query()
            .table_name(&self.table_name)
            .key_condition_expression("PK = :pk AND begins_with(SK, :sk_prefix)")
            .expression_attribute_values(":pk", AttributeValue::S(format!("CLASS#{}", class_id)))
            .expression_attribute_values(":sk_prefix", AttributeValue::S("ABILITY#".to_string()))
            .send()
            .await
            .context("Failed to query class abilities")?;

        let mut abilities = Vec::new();
        for item in result.items() {
            if let (Some(ability_id), Some(level)) = (
                item.get("ability_id").and_then(|v| v.as_s().ok()),
                self.get_unlock_level(item).ok(),
            ) {
                abilities.push((ability_id.to_string(), level));
            }
        }

        Ok(abilities)
    }
}
