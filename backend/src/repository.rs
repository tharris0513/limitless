use crate::config_export::{ClassAbilityMapping, ClassWithAbilities, GameConfigExport};
use crate::models::{Ability, Character, CharacterAbility, CharacterStats, Class, User};
use anyhow::{Context, Result};
use aws_sdk_dynamodb::types::AttributeValue;
use aws_sdk_dynamodb::Client as DynamoDbClient;
use chrono::Utc;
use std::collections::HashMap;
use uuid::Uuid;

pub struct UserRepository {
    pub client: DynamoDbClient,
    pub table_name: String,
}

impl UserRepository {
    pub fn new(client: DynamoDbClient, table_name: String) -> Self {
        Self { client, table_name }
    }

    // Find user by Discord ID, create if not exists
    pub async fn find_or_create_by_discord_id(
        &self,
        discord_id: &str,
        discord_name: &str,
    ) -> Result<User> {
        // Try to find existing user
        if let Ok(user) = self.find_by_discord_id(discord_id).await {
            return Ok(user);
        }

        // Create new user if not found
        self.create_user(discord_id, discord_name).await
    }

    pub async fn find_by_discord_id(&self, discord_id: &str) -> Result<User> {
        // Query the GSI (discord_id-index)
        let result = self
            .client
            .query()
            .table_name(&self.table_name)
            .index_name("discord_id-index")
            .key_condition_expression("discord_id = :discord_id")
            .expression_attribute_values(":discord_id", AttributeValue::S(discord_id.to_string()))
            .send()
            .await
            .context("Failed to query user by discord_id")?;

        let items = result.items();
        if items.is_empty() {
            return Err(anyhow::anyhow!("User not found"));
        }

        self.parse_user(&items[0])
    }

    pub async fn find_by_id(&self, id: &str) -> Result<User> {
        let result = self
            .client
            .get_item()
            .table_name(&self.table_name)
            .key("PK", AttributeValue::S(format!("USER#{}", id)))
            .key("SK", AttributeValue::S("PROFILE".to_string()))
            .send()
            .await
            .context("Failed to get user by id")?;

        match result.item() {
            Some(item) => self.parse_user(item),
            None => Err(anyhow::anyhow!("User not found")),
        }
    }

    // Admin only - get all users
    pub async fn get_all_users(&self) -> Result<Vec<User>> {
        let result = self
            .client
            .query()
            .table_name(&self.table_name)
            .index_name("entity-type-index")
            .key_condition_expression("entity_type = :entity_type")
            .expression_attribute_values(":entity_type", AttributeValue::S("USER".to_string()))
            .scan_index_forward(false)
            .send()
            .await
            .context("Failed to query all users")?;

        let mut users = Vec::new();
        for item in result.items() {
            users.push(self.parse_user(item)?);
        }

        Ok(users)
    }

    async fn create_user(&self, discord_id: &str, discord_name: &str) -> Result<User> {
        let id = Uuid::new_v4().to_string();
        let created_at = Utc::now().to_rfc3339();

        let mut item = HashMap::new();
        item.insert("PK".to_string(), AttributeValue::S(format!("USER#{}", id)));
        item.insert("SK".to_string(), AttributeValue::S("PROFILE".to_string()));
        item.insert(
            "entity_type".to_string(),
            AttributeValue::S("USER".to_string()),
        );
        item.insert("id".to_string(), AttributeValue::S(id.clone()));
        item.insert(
            "discord_id".to_string(),
            AttributeValue::S(discord_id.to_string()),
        );
        item.insert(
            "discord_name".to_string(),
            AttributeValue::S(discord_name.to_string()),
        );
        item.insert("admin".to_string(), AttributeValue::Bool(false));
        item.insert("banned".to_string(), AttributeValue::Bool(false));
        item.insert(
            "created_at".to_string(),
            AttributeValue::S(created_at.clone()),
        );

        self.client
            .put_item()
            .table_name(&self.table_name)
            .set_item(Some(item))
            .send()
            .await
            .context("Failed to create user")?;

        Ok(User {
            id,
            discord_id: discord_id.to_string(),
            discord_name: discord_name.to_string(),
            username: None,
            date_of_birth: None,
            admin: false,
            banned: false,
            created_at,
        })
    }

    fn parse_user(&self, item: &HashMap<String, AttributeValue>) -> Result<User> {
        Ok(User {
            id: item
                .get("id")
                .and_then(|v| v.as_s().ok())
                .context("Missing id")?
                .to_string(),
            discord_id: item
                .get("discord_id")
                .and_then(|v| v.as_s().ok())
                .context("Missing discord_id")?
                .to_string(),
            discord_name: item
                .get("discord_name")
                .and_then(|v| v.as_s().ok())
                .context("Missing discord_name")?
                .to_string(),
            username: item
                .get("username")
                .and_then(|v| v.as_s().ok())
                .map(|s| s.to_string()),
            date_of_birth: item
                .get("date_of_birth")
                .and_then(|v| v.as_s().ok())
                .map(|s| s.to_string()),
            admin: item
                .get("admin")
                .and_then(|v| v.as_bool().ok())
                .copied()
                .unwrap_or(false),
            banned: item
                .get("banned")
                .and_then(|v| v.as_bool().ok())
                .copied()
                .unwrap_or(false),
            created_at: item
                .get("created_at")
                .and_then(|v| v.as_s().ok())
                .context("Missing created_at")?
                .to_string(),
        })
    }

    pub async fn update_user(
        &self,
        user_id: &str,
        username: Option<&str>,
        date_of_birth: Option<&str>,
    ) -> Result<User> {
        let mut update_expression = String::from("SET");
        let mut expression_attribute_values = HashMap::new();
        let mut update_parts = Vec::new();

        if let Some(username) = username {
            update_parts.push(" username = :username");
            expression_attribute_values.insert(
                ":username".to_string(),
                AttributeValue::S(username.to_string()),
            );
        }

        if let Some(dob) = date_of_birth {
            update_parts.push(" date_of_birth = :dob");
            expression_attribute_values
                .insert(":dob".to_string(), AttributeValue::S(dob.to_string()));
        }

        if update_parts.is_empty() {
            return self.find_by_id(user_id).await;
        }

        update_expression.push_str(&update_parts.join(","));

        self.client
            .update_item()
            .table_name(&self.table_name)
            .key("PK", AttributeValue::S(format!("USER#{}", user_id)))
            .key("SK", AttributeValue::S("PROFILE".to_string()))
            .update_expression(update_expression)
            .set_expression_attribute_values(Some(expression_attribute_values))
            .return_values(aws_sdk_dynamodb::types::ReturnValue::AllNew)
            .send()
            .await
            .context("Failed to update user")?;

        // Fetch and return updated user
        self.find_by_id(user_id).await
    }

    pub async fn delete_user(&self, user_id: &str) -> Result<()> {
        // Delete user profile
        self.client
            .delete_item()
            .table_name(&self.table_name)
            .key("PK", AttributeValue::S(format!("USER#{}", user_id)))
            .key("SK", AttributeValue::S("PROFILE".to_string()))
            .send()
            .await
            .context("Failed to delete user profile")?;

        // Also delete all characters for this user
        let characters = self.get_user_characters(user_id).await?;
        for character in characters {
            self.client
                .delete_item()
                .table_name(&self.table_name)
                .key("PK", AttributeValue::S(format!("USER#{}", user_id)))
                .key("SK", AttributeValue::S(format!("CHAR#{}", character.id)))
                .send()
                .await
                .context("Failed to delete user character")?;
        }

        tracing::info!("Deleted user {} and their characters", user_id);
        Ok(())
    }

    pub async fn update_user_ban_status(&self, user_id: &str, banned: bool) -> Result<User> {
        self.client
            .update_item()
            .table_name(&self.table_name)
            .key("PK", AttributeValue::S(format!("USER#{}", user_id)))
            .key("SK", AttributeValue::S("PROFILE".to_string()))
            .update_expression("SET banned = :banned")
            .expression_attribute_values(":banned", AttributeValue::Bool(banned))
            .return_values(aws_sdk_dynamodb::types::ReturnValue::AllNew)
            .send()
            .await
            .context("Failed to update user ban status")?;

        // Fetch and return updated user
        self.find_by_id(user_id).await
    }

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

    fn parse_character(&self, item: &HashMap<String, AttributeValue>) -> Result<Character> {
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
    async fn unlock_character_abilities(
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

    fn parse_class(&self, item: &HashMap<String, AttributeValue>) -> Result<Class> {
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

    fn parse_ability_from_class_item(
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

        let get_f64 = |key: &str| -> Option<f64> {
            item.get(key)
                .and_then(|v| v.as_n().ok())
                .and_then(|s| s.parse::<f64>().ok())
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

    fn get_unlock_level(&self, item: &HashMap<String, AttributeValue>) -> Result<i64> {
        item.get("unlock_level")
            .and_then(|v| v.as_n().ok())
            .and_then(|s| s.parse::<i64>().ok())
            .context("Missing unlock_level")
    }

    fn parse_character_ability(
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

    fn parse_ability(&self, item: &HashMap<String, AttributeValue>) -> Result<Ability> {
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

        let get_f64 = |key: &str| -> Option<f64> {
            item.get(key)
                .and_then(|v| v.as_n().ok())
                .and_then(|s| s.parse::<f64>().ok())
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

    // Get class abilities with their full data (helper for export)
    async fn get_class_abilities_full(&self, class_id: &str) -> Result<Vec<(String, i64)>> {
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
