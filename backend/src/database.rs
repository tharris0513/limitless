use anyhow::Result;
use aws_sdk_dynamodb::Client as DynamoDbClient;

#[derive(Clone)]
pub struct Database {
    pub client: DynamoDbClient,
    pub table_name: String,
}

impl Database {
    pub async fn new(table_name: Option<String>) -> Result<Self> {
        let config = aws_config::load_from_env().await;
        let client = DynamoDbClient::new(&config);
        
        let table_name = table_name.unwrap_or_else(|| {
            std::env::var("DYNAMODB_TABLE_NAME")
                .unwrap_or_else(|_| "limitless-game".to_string())
        });

        Ok(Database { client, table_name })
    }
}

pub async fn init_database() -> Result<DynamoDbClient> {
    let db = Database::new(None).await?;
    Ok(db.client)
}
