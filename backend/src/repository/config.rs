use anyhow::{Context, Result};
use aws_sdk_dynamodb::types::AttributeValue;
use std::collections::HashMap;

use super::UserRepository;

impl UserRepository {
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
