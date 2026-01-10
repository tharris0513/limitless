use aws_sdk_dynamodb::Client as DynamoDbClient;

// Include all the repository modules
#[path = "ability.rs"]
mod ability;
#[path = "adventure.rs"]
mod adventure;
#[path = "character.rs"]
mod character;
#[path = "class.rs"]
mod class;
#[path = "config.rs"]
mod config;
#[path = "creature.rs"]
mod creature;
#[path = "location.rs"]
mod location;
#[path = "rollover.rs"]
mod rollover;
#[path = "user.rs"]
mod user;

pub struct UserRepository {
    pub client: DynamoDbClient,
    pub table_name: String,
}

impl UserRepository {
    pub fn new(client: DynamoDbClient, table_name: String) -> Self {
        Self { client, table_name }
    }
}
