use crate::config_export::{ClassAbilityMapping, ClassWithAbilities, GameConfigExport};
use anyhow::{Context, Result};

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
}
