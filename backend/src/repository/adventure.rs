use crate::models::Adventure;
use anyhow::{Context, Result};
use aws_sdk_dynamodb::types::AttributeValue;
use std::collections::HashMap;

use super::UserRepository;

impl UserRepository {
    // Create a new adventure
    pub async fn create_adventure(&self, adventure: Adventure) -> Result<Adventure> {
        let mut item = HashMap::new();
        item.insert(
            "PK".to_string(),
            AttributeValue::S(format!("ADVENTURE#{}", adventure.id)),
        );
        item.insert("SK".to_string(), AttributeValue::S("METADATA".to_string()));
        item.insert(
            "entity_type".to_string(),
            AttributeValue::S("ADVENTURE".to_string()),
        );
        item.insert("id".to_string(), AttributeValue::S(adventure.id.clone()));
        item.insert(
            "name".to_string(),
            AttributeValue::S(adventure.name.clone()),
        );
        item.insert(
            "description".to_string(),
            AttributeValue::S(adventure.description.clone()),
        );
        item.insert(
            "required_level".to_string(),
            AttributeValue::N(adventure.required_level.to_string()),
        );
        item.insert(
            "adventure_type".to_string(),
            AttributeValue::S(adventure.adventure_type.clone()),
        );
        item.insert(
            "experience_reward".to_string(),
            AttributeValue::N(adventure.experience_reward.to_string()),
        );
        item.insert(
            "created_at".to_string(),
            AttributeValue::S(adventure.created_at.clone()),
        );

        self.client
            .put_item()
            .table_name(&self.table_name)
            .set_item(Some(item))
            .send()
            .await
            .context("Failed to create adventure")?;

        Ok(adventure)
    }

    // Get an adventure by ID
    pub async fn get_adventure(&self, adventure_id: &str) -> Result<Adventure> {
        let result = self
            .client
            .get_item()
            .table_name(&self.table_name)
            .key(
                "PK",
                AttributeValue::S(format!("ADVENTURE#{}", adventure_id)),
            )
            .key("SK", AttributeValue::S("METADATA".to_string()))
            .send()
            .await
            .context("Failed to get adventure")?;

        match result.item() {
            Some(item) => self.parse_adventure(item),
            None => Err(anyhow::anyhow!("Adventure not found")),
        }
    }

    // Get all adventures
    pub async fn get_all_adventures(&self) -> Result<Vec<Adventure>> {
        let result = self
            .client
            .scan()
            .table_name(&self.table_name)
            .filter_expression("entity_type = :entity_type")
            .expression_attribute_values(":entity_type", AttributeValue::S("ADVENTURE".to_string()))
            .send()
            .await
            .context("Failed to scan adventures")?;

        let mut adventures = Vec::new();
        for item in result.items() {
            adventures.push(self.parse_adventure(item)?);
        }

        Ok(adventures)
    }

    // Update an adventure
    pub async fn update_adventure(
        &self,
        adventure_id: &str,
        adventure: Adventure,
    ) -> Result<Adventure> {
        // Delete old adventure and create new one
        self.delete_adventure(adventure_id).await?;
        self.create_adventure(adventure).await
    }

    // Delete an adventure
    pub async fn delete_adventure(&self, adventure_id: &str) -> Result<()> {
        self.client
            .delete_item()
            .table_name(&self.table_name)
            .key(
                "PK",
                AttributeValue::S(format!("ADVENTURE#{}", adventure_id)),
            )
            .key("SK", AttributeValue::S("METADATA".to_string()))
            .send()
            .await
            .context("Failed to delete adventure")?;

        Ok(())
    }

    pub(crate) fn parse_adventure(
        &self,
        item: &HashMap<String, AttributeValue>,
    ) -> Result<Adventure> {
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

        Ok(Adventure {
            id: get_string("id")?,
            name: get_string("name")?,
            description: get_string("description")?,
            required_level: get_i64("required_level")?,
            adventure_type: get_string("adventure_type")?,
            experience_reward: get_i64("experience_reward")?,
            created_at: get_string("created_at")?,
        })
    }
}
