use crate::models::Creature;
use anyhow::{Context, Result};
use aws_sdk_dynamodb::types::AttributeValue;
use std::collections::HashMap;

use super::UserRepository;

impl UserRepository {
    // Create a new creature
    pub async fn create_creature(&self, creature: Creature) -> Result<Creature> {
        let mut item = HashMap::new();
        item.insert(
            "PK".to_string(),
            AttributeValue::S(format!("CREATURE#{}", creature.id)),
        );
        item.insert("SK".to_string(), AttributeValue::S("METADATA".to_string()));
        item.insert(
            "entity_type".to_string(),
            AttributeValue::S("CREATURE".to_string()),
        );
        item.insert("id".to_string(), AttributeValue::S(creature.id.clone()));
        item.insert("name".to_string(), AttributeValue::S(creature.name.clone()));
        item.insert(
            "introduction_text".to_string(),
            AttributeValue::S(creature.introduction_text.clone()),
        );
        item.insert(
            "level".to_string(),
            AttributeValue::N(creature.level.to_string()),
        );
        item.insert(
            "health".to_string(),
            AttributeValue::N(creature.health.to_string()),
        );
        item.insert(
            "might".to_string(),
            AttributeValue::N(creature.might.to_string()),
        );
        item.insert(
            "defense".to_string(),
            AttributeValue::N(creature.defense.to_string()),
        );
        item.insert(
            "magic".to_string(),
            AttributeValue::N(creature.magic.to_string()),
        );
        item.insert(
            "resistance".to_string(),
            AttributeValue::N(creature.resistance.to_string()),
        );
        item.insert(
            "agility".to_string(),
            AttributeValue::N(creature.agility.to_string()),
        );
        item.insert(
            "experience_reward".to_string(),
            AttributeValue::N(creature.experience_reward.to_string()),
        );
        item.insert(
            "creature_type".to_string(),
            AttributeValue::S(creature.creature_type.clone()),
        );
        item.insert(
            "created_at".to_string(),
            AttributeValue::S(creature.created_at.clone()),
        );

        self.client
            .put_item()
            .table_name(&self.table_name)
            .set_item(Some(item))
            .send()
            .await
            .context("Failed to create creature")?;

        Ok(creature)
    }

    // Get a creature by ID
    pub async fn get_creature(&self, creature_id: &str) -> Result<Creature> {
        let result = self
            .client
            .get_item()
            .table_name(&self.table_name)
            .key("PK", AttributeValue::S(format!("CREATURE#{}", creature_id)))
            .key("SK", AttributeValue::S("METADATA".to_string()))
            .send()
            .await
            .context("Failed to get creature")?;

        match result.item() {
            Some(item) => self.parse_creature(item),
            None => Err(anyhow::anyhow!("Creature not found")),
        }
    }

    // Get all creatures
    pub async fn get_all_creatures(&self) -> Result<Vec<Creature>> {
        let result = self
            .client
            .scan()
            .table_name(&self.table_name)
            .filter_expression("entity_type = :entity_type")
            .expression_attribute_values(":entity_type", AttributeValue::S("CREATURE".to_string()))
            .send()
            .await
            .context("Failed to scan creatures")?;

        let mut creatures = Vec::new();
        for item in result.items() {
            creatures.push(self.parse_creature(item)?);
        }

        Ok(creatures)
    }

    // Update a creature
    pub async fn update_creature(&self, creature_id: &str, creature: Creature) -> Result<Creature> {
        // Delete old creature and create new one
        self.delete_creature(creature_id).await?;
        self.create_creature(creature).await
    }

    // Delete a creature
    pub async fn delete_creature(&self, creature_id: &str) -> Result<()> {
        self.client
            .delete_item()
            .table_name(&self.table_name)
            .key("PK", AttributeValue::S(format!("CREATURE#{}", creature_id)))
            .key("SK", AttributeValue::S("METADATA".to_string()))
            .send()
            .await
            .context("Failed to delete creature")?;

        Ok(())
    }

    pub(crate) fn parse_creature(
        &self,
        item: &HashMap<String, AttributeValue>,
    ) -> Result<Creature> {
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

        Ok(Creature {
            id: get_string("id")?,
            name: get_string("name")?,
            introduction_text: get_string("introduction_text")
                .or_else(|_| get_string("description"))?, // Backward compatibility with old field name
            level: get_i64("level")?,
            health: get_i64("health")?,
            might: get_i64("might")?,

            defense: get_i64("defense")?,
            magic: get_i64("magic")?,
            resistance: get_i64("resistance")?,
            agility: get_i64("agility")?,
            experience_reward: get_i64("experience_reward")?,
            creature_type: get_string("creature_type")?,
            created_at: get_string("created_at")?,
        })
    }
}
