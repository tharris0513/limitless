use aws_sdk_dynamodb::Client as DynamoDbClient;

// Include all the repository modules
#[path = "user.rs"]
mod user;
#[path = "character.rs"]
mod character;
#[path = "class.rs"]
mod class;
#[path = "ability.rs"]
mod ability;
#[path = "config.rs"]
mod config;

pub struct UserRepository {
    pub client: DynamoDbClient,
    pub table_name: String,
}

impl UserRepository {
    pub fn new(client: DynamoDbClient, table_name: String) -> Self {
        Self { client, table_name }
    }
}
