use aws_sdk_dynamodb::Client as DynamoDbClient;
mod character;
mod config;
mod rollover;
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
