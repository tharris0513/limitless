use crate::models::User;
use anyhow::{Context, Result};
use aws_sdk_dynamodb::types::AttributeValue;
use std::collections::HashMap;
use uuid::Uuid;
use chrono::Utc;

use super::UserRepository;

impl UserRepository {
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

    pub(crate) fn parse_user(&self, item: &HashMap<String, AttributeValue>) -> Result<User> {
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
}
