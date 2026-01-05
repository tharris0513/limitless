use crate::config_export::{ClassAbilityMapping, ClassWithAbilities, GameConfigExport};
use anyhow::{Context, Result};
use aws_sdk_dynamodb::types::AttributeValue;
use std::collections::HashMap;

use super::UserRepository;

impl UserRepository {
    // Export all game configuration (classes and abilities)
    pub async fn export_game_config(&self) -> Result<GameConfigExport> {
        let classes = self.get_all_classes().await?;
        let abilities = self.get_all_abilities().await?;

        let mut classes_with_abilities = Vec::new();
        for class in classes {
            let class_ability_ids = self.get_class_abilities_full(&class.id).await?;
            let ability_mappings: Vec<ClassAbilityMapping> = class_ability_ids
                .into_iter()
                .map(|(ability_id, unlock_level)| ClassAbilityMapping {
                    ability_id,
                    unlock_level,
                })
                .collect();

            classes_with_abilities.push(ClassWithAbilities {
                class,
                abilities: ability_mappings,
            });
        }

        Ok(GameConfigExport::new(classes_with_abilities, abilities))
    }

    // Import game configuration (classes and abilities)
    pub async fn import_game_config(&self, config: GameConfigExport) -> Result<()> {
        // First, import all abilities
        for ability in config.abilities {
            // Delete if exists, then create
            let _ = self.delete_ability(&ability.id).await;
            self.create_ability(ability).await?;
        }

        // Then import classes and their ability mappings
        for class_with_abilities in config.classes {
            // Delete if exists, then create
            let _ = self.delete_class(&class_with_abilities.class.id).await;
            self.create_class(class_with_abilities.class.clone())
                .await?;

            // Add abilities to class
            for mapping in class_with_abilities.abilities {
                self.add_class_ability(
                    &class_with_abilities.class.id,
                    &mapping.ability_id,
                    mapping.unlock_level,
                )
                .await?;
            }
        }

        Ok(())
    }

    // Health check - verify database connectivity
    pub async fn health_check(&self) -> Result<()> {
        self.client
            .list_tables()
            .limit(1)
            .send()
            .await
            .context("Failed to connect to DynamoDB")?;
        Ok(())
    }

    // Get maintenance mode status
    pub async fn get_maintenance_mode(&self) -> Result<bool> {
        let result = self
            .client
            .get_item()
            .table_name(&self.table_name)
            .key("PK", AttributeValue::S("CONFIG#MAINTENANCE".to_string()))
            .key("SK", AttributeValue::S("STATUS".to_string()))
            .send()
            .await;

        match result {
            Ok(output) => {
                if let Some(item) = output.item() {
                    let enabled = item
                        .get("enabled")
                        .and_then(|v| v.as_bool().ok())
                        .copied()
                        .unwrap_or(false);
                    Ok(enabled)
                } else {
                    Ok(false) // Default to not in maintenance mode
                }
            }
            Err(_) => Ok(false), // If item doesn't exist, not in maintenance mode
        }
    }

    // Set maintenance mode status
    pub async fn set_maintenance_mode(&self, enabled: bool) -> Result<()> {
        let mut item = HashMap::new();
        item.insert(
            "PK".to_string(),
            AttributeValue::S("CONFIG#MAINTENANCE".to_string()),
        );
        item.insert("SK".to_string(), AttributeValue::S("STATUS".to_string()));
        item.insert(
            "entity_type".to_string(),
            AttributeValue::S("CONFIG".to_string()),
        );
        item.insert("enabled".to_string(), AttributeValue::Bool(enabled));
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
            .context("Failed to set maintenance mode")?;

        Ok(())
    }
}
