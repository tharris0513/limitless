use crate::models::{Character, CharacterStats, User};
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

    pub async fn create_character(&self, user_id: &str, name: &str) -> Result<Character> {
        let character_id = Uuid::new_v4().to_string();
        let created_at = Utc::now().to_rfc3339();
        let last_played = created_at.clone();

        let default_stats = CharacterStats {
            might: 10,
            defense: 10,
            magic: 10,
            resistance: 10,
            agility: 10,
            adventures: 5,
            max_adventures: 5,
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
        item.insert("level".to_string(), AttributeValue::N("1".to_string()));
        item.insert("health".to_string(), AttributeValue::N("100".to_string()));
        item.insert(
            "max_health".to_string(),
            AttributeValue::N("100".to_string()),
        );
        item.insert("mana".to_string(), AttributeValue::N("50".to_string()));
        item.insert("max_mana".to_string(), AttributeValue::N("50".to_string()));
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
            AttributeValue::N(default_stats.might.to_string()),
        );
        item.insert(
            "defense".to_string(),
            AttributeValue::N(default_stats.defense.to_string()),
        );
        item.insert(
            "magic".to_string(),
            AttributeValue::N(default_stats.magic.to_string()),
        );
        item.insert(
            "resistance".to_string(),
            AttributeValue::N(default_stats.resistance.to_string()),
        );
        item.insert(
            "agility".to_string(),
            AttributeValue::N(default_stats.agility.to_string()),
        );
        item.insert(
            "adventures".to_string(),
            AttributeValue::N(default_stats.adventures.to_string()),
        );
        item.insert(
            "max_adventures".to_string(),
            AttributeValue::N(default_stats.max_adventures.to_string()),
        );

        self.client
            .put_item()
            .table_name(&self.table_name)
            .set_item(Some(item))
            .send()
            .await
            .context("Failed to create character")?;

        Ok(Character {
            id: character_id,
            user_id: user_id.to_string(),
            name: name.to_string(),
            level: 1,
            health: 100,
            max_health: 100,
            mana: 50,
            max_mana: 50,
            experience: 0,
            experience_to_next: 100,
            stats: default_stats,
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
                max_adventures: get_i64("max_adventures")?,
            },
            location: get_string("location")?,
            created_at: get_string("created_at")?,
            last_played: get_string("last_played")?,
        })
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
