use crate::models::Class;
use anyhow::{Context, Result};
use aws_sdk_dynamodb::types::AttributeValue;
use std::collections::HashMap;

use super::UserRepository;

impl UserRepository {
    // Get a class by ID
    pub async fn get_class(&self, class_id: &str) -> Result<Class> {
        let result = self
            .client
            .get_item()
            .table_name(&self.table_name)
            .key("PK", AttributeValue::S(format!("CLASS#{}", class_id)))
            .key("SK", AttributeValue::S("METADATA".to_string()))
            .send()
            .await
            .context("Failed to get class")?;

        match result.item() {
            Some(item) => self.parse_class(item),
            None => Err(anyhow::anyhow!("Class not found")),
        }
    }

    // Get all available classes
    pub async fn get_all_classes(&self) -> Result<Vec<Class>> {
        let result = self
            .client
            .scan()
            .table_name(&self.table_name)
            .filter_expression("entity_type = :entity_type")
            .expression_attribute_values(":entity_type", AttributeValue::S("CLASS".to_string()))
            .send()
            .await
            .context("Failed to scan classes")?;

        let mut classes = Vec::new();
        for item in result.items() {
            classes.push(self.parse_class(item)?);
        }

        Ok(classes)
    }

    pub(crate) fn parse_class(&self, item: &HashMap<String, AttributeValue>) -> Result<Class> {
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

        Ok(Class {
            id: get_string("id")?,
            name: get_string("name")?,
            description: get_string("description")?,
            starting_might: get_i64("starting_might")?,
            starting_defense: get_i64("starting_defense")?,
            starting_magic: get_i64("starting_magic")?,
            starting_resistance: get_i64("starting_resistance")?,
            starting_agility: get_i64("starting_agility")?,
            starting_health: get_i64("starting_health")?,
            starting_mana: get_i64("starting_mana")?,
        })
    }

    // Create a new class
    pub async fn create_class(&self, class: Class) -> Result<Class> {
        let mut item = HashMap::new();
        item.insert(
            "PK".to_string(),
            AttributeValue::S(format!("CLASS#{}", class.id)),
        );
        item.insert("SK".to_string(), AttributeValue::S("METADATA".to_string()));
        item.insert(
            "entity_type".to_string(),
            AttributeValue::S("CLASS".to_string()),
        );
        item.insert("id".to_string(), AttributeValue::S(class.id.clone()));
        item.insert("name".to_string(), AttributeValue::S(class.name.clone()));
        item.insert(
            "description".to_string(),
            AttributeValue::S(class.description.clone()),
        );
        item.insert(
            "starting_might".to_string(),
            AttributeValue::N(class.starting_might.to_string()),
        );
        item.insert(
            "starting_defense".to_string(),
            AttributeValue::N(class.starting_defense.to_string()),
        );
        item.insert(
            "starting_magic".to_string(),
            AttributeValue::N(class.starting_magic.to_string()),
        );
        item.insert(
            "starting_resistance".to_string(),
            AttributeValue::N(class.starting_resistance.to_string()),
        );
        item.insert(
            "starting_agility".to_string(),
            AttributeValue::N(class.starting_agility.to_string()),
        );
        item.insert(
            "starting_health".to_string(),
            AttributeValue::N(class.starting_health.to_string()),
        );
        item.insert(
            "starting_mana".to_string(),
            AttributeValue::N(class.starting_mana.to_string()),
        );

        self.client
            .put_item()
            .table_name(&self.table_name)
            .set_item(Some(item))
            .send()
            .await
            .context("Failed to create class")?;

        Ok(class)
    }

    // Update a class
    pub async fn update_class(&self, class_id: &str, class: Class) -> Result<Class> {
        // Delete old class and create new one (simpler than building update expression)
        self.delete_class(class_id).await?;
        self.create_class(class).await
    }

    // Delete a class
    pub async fn delete_class(&self, class_id: &str) -> Result<()> {
        // Delete class metadata
        self.client
            .delete_item()
            .table_name(&self.table_name)
            .key("PK", AttributeValue::S(format!("CLASS#{}", class_id)))
            .key("SK", AttributeValue::S("METADATA".to_string()))
            .send()
            .await
            .context("Failed to delete class")?;

        // Delete all abilities for this class
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

        for item in result.items() {
            if let Some(sk) = item.get("SK").and_then(|v| v.as_s().ok()) {
                self.client
                    .delete_item()
                    .table_name(&self.table_name)
                    .key("PK", AttributeValue::S(format!("CLASS#{}", class_id)))
                    .key("SK", AttributeValue::S(sk.to_string()))
                    .send()
                    .await
                    .context("Failed to delete class ability")?;
            }
        }

        Ok(())
    }

    // Add ability to class
    pub async fn add_class_ability(
        &self,
        class_id: &str,
        ability_id: &str,
        unlock_level: i64,
    ) -> Result<()> {
        // First get the ability details
        let ability = self.get_ability(ability_id).await?;

        let mut item = HashMap::new();
        item.insert(
            "PK".to_string(),
            AttributeValue::S(format!("CLASS#{}", class_id)),
        );
        item.insert(
            "SK".to_string(),
            AttributeValue::S(format!("ABILITY#{}", ability_id)),
        );
        item.insert(
            "entity_type".to_string(),
            AttributeValue::S("CLASS_ABILITY".to_string()),
        );
        item.insert(
            "class_id".to_string(),
            AttributeValue::S(class_id.to_string()),
        );
        item.insert(
            "ability_id".to_string(),
            AttributeValue::S(ability_id.to_string()),
        );
        item.insert(
            "ability_name".to_string(),
            AttributeValue::S(ability.name.clone()),
        );
        item.insert(
            "ability_description".to_string(),
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

        item.insert(
            "unlock_level".to_string(),
            AttributeValue::N(unlock_level.to_string()),
        );

        self.client
            .put_item()
            .table_name(&self.table_name)
            .set_item(Some(item))
            .send()
            .await
            .context("Failed to add ability to class")?;

        Ok(())
    }

    // Remove ability from class
    pub async fn remove_class_ability(&self, class_id: &str, ability_id: &str) -> Result<()> {
        self.client
            .delete_item()
            .table_name(&self.table_name)
            .key("PK", AttributeValue::S(format!("CLASS#{}", class_id)))
            .key("SK", AttributeValue::S(format!("ABILITY#{}", ability_id)))
            .send()
            .await
            .context("Failed to remove ability from class")?;

        Ok(())
    }
}
