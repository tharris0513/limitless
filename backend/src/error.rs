use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde::Serialize;
use std::collections::HashMap;

// Standard API error response format
#[derive(Serialize)]
pub struct ApiError {
    pub error: String,
    pub message: String,
    pub status_code: u16,
    pub timestamp: String,
    pub details: Option<HashMap<String, serde_json::Value>>,
}

// Application-specific error types
#[derive(Debug)]
pub enum AppError {
    // Database errors
    DatabaseError(aws_sdk_dynamodb::Error),
    DatabaseConnectionError(String),

    // Authentication errors
    AuthenticationError(String),
    AuthorizationError(String),
    Forbidden,
    TokenExpired,
    InvalidToken,

    // Validation errors
    ValidationError(String),
    InvalidInput(HashMap<String, String>),

    // Business logic errors
    UserNotFound(String),
    CharacterNotFound(String),
    DuplicateResource(String),

    // External service errors
    DiscordApiError(String),

    // Generic errors
    InternalServerError(String),
    BadRequest(String),
    NotFound(String),
    Conflict(String),
}

impl AppError {
    fn status_code(&self) -> StatusCode {
        match self {
            AppError::DatabaseError(_) => StatusCode::INTERNAL_SERVER_ERROR,
            AppError::DatabaseConnectionError(_) => StatusCode::SERVICE_UNAVAILABLE,

            AppError::AuthenticationError(_) => StatusCode::UNAUTHORIZED,
            AppError::AuthorizationError(_) => StatusCode::FORBIDDEN,
            AppError::Forbidden => StatusCode::FORBIDDEN,
            AppError::TokenExpired => StatusCode::UNAUTHORIZED,
            AppError::InvalidToken => StatusCode::UNAUTHORIZED,

            AppError::ValidationError(_) => StatusCode::BAD_REQUEST,
            AppError::InvalidInput(_) => StatusCode::BAD_REQUEST,

            AppError::UserNotFound(_) => StatusCode::NOT_FOUND,
            AppError::CharacterNotFound(_) => StatusCode::NOT_FOUND,
            AppError::DuplicateResource(_) => StatusCode::CONFLICT,

            AppError::DiscordApiError(_) => StatusCode::BAD_GATEWAY,

            AppError::InternalServerError(_) => StatusCode::INTERNAL_SERVER_ERROR,
            AppError::BadRequest(_) => StatusCode::BAD_REQUEST,
            AppError::NotFound(_) => StatusCode::NOT_FOUND,
            AppError::Conflict(_) => StatusCode::CONFLICT,
        }
    }

    fn error_type(&self) -> &'static str {
        match self {
            AppError::DatabaseError(_) | AppError::DatabaseConnectionError(_) => "DATABASE_ERROR",
            AppError::AuthenticationError(_) | AppError::TokenExpired | AppError::InvalidToken => {
                "AUTHENTICATION_ERROR"
            }
            AppError::AuthorizationError(_) => "AUTHORIZATION_ERROR",
            AppError::Forbidden => "FORBIDDEN",
            AppError::ValidationError(_) | AppError::InvalidInput(_) => "VALIDATION_ERROR",
            AppError::UserNotFound(_) | AppError::CharacterNotFound(_) => "RESOURCE_NOT_FOUND",
            AppError::DuplicateResource(_) => "DUPLICATE_RESOURCE",
            AppError::DiscordApiError(_) => "EXTERNAL_SERVICE_ERROR",
            AppError::InternalServerError(_) => "INTERNAL_SERVER_ERROR",
            AppError::BadRequest(_) => "BAD_REQUEST",
            AppError::NotFound(_) => "NOT_FOUND",
            AppError::Conflict(_) => "CONFLICT",
        }
    }

    fn message(&self) -> String {
        match self {
            AppError::DatabaseError(_) => "Database operation failed".to_string(),
            AppError::DatabaseConnectionError(msg) => {
                format!("Database connection failed: {}", msg)
            }

            AppError::AuthenticationError(msg) => msg.clone(),
            AppError::AuthorizationError(msg) => msg.clone(),
            AppError::Forbidden => "Access to this resource is forbidden".to_string(),
            AppError::TokenExpired => "Authentication token has expired".to_string(),
            AppError::InvalidToken => "Invalid authentication token".to_string(),

            AppError::ValidationError(msg) => msg.clone(),
            AppError::InvalidInput(_) => "Invalid input provided".to_string(),

            AppError::UserNotFound(id) => format!("User with ID '{}' not found", id),
            AppError::CharacterNotFound(id) => format!("Character with ID '{}' not found", id),
            AppError::DuplicateResource(msg) => format!("Resource already exists: {}", msg),

            AppError::DiscordApiError(msg) => format!("Discord API error: {}", msg),

            AppError::InternalServerError(msg) => format!("Internal server error: {}", msg),
            AppError::BadRequest(msg) => msg.clone(),
            AppError::NotFound(msg) => msg.clone(),
            AppError::Conflict(msg) => msg.clone(),
        }
    }

    fn details(&self) -> Option<HashMap<String, serde_json::Value>> {
        match self {
            AppError::InvalidInput(fields) => {
                let mut details = HashMap::new();
                for (field, error) in fields {
                    details.insert(field.clone(), serde_json::Value::String(error.clone()));
                }
                Some(details)
            }
            AppError::DatabaseError(e) => {
                let mut details = HashMap::new();
                details.insert(
                    "error_type".to_string(),
                    serde_json::Value::String("database".to_string()),
                );
                details.insert(
                    "database_error".to_string(),
                    serde_json::Value::String(e.to_string()),
                );
                Some(details)
            }
            _ => None,
        }
    }
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let status = self.status_code();

        // Log the error (with different levels based on severity)
        match &self {
            AppError::InternalServerError(_) | AppError::DatabaseError(_) => {
                tracing::error!("Application error: {:?}", self);
            }
            AppError::DiscordApiError(_) | AppError::DatabaseConnectionError(_) => {
                tracing::warn!("Application warning: {:?}", self);
            }
            _ => {
                tracing::info!("Application info: {:?}", self);
            }
        }

        let api_error = ApiError {
            error: self.error_type().to_string(),
            message: self.message(),
            status_code: status.as_u16(),
            timestamp: chrono::Utc::now().to_rfc3339(),
            details: self.details(),
        };

        (status, Json(api_error)).into_response()
    }
}

// Implement From traits for common error types
impl From<aws_sdk_dynamodb::Error> for AppError {
    fn from(err: aws_sdk_dynamodb::Error) -> Self {
        AppError::DatabaseError(err)
    }
}

impl From<anyhow::Error> for AppError {
    fn from(err: anyhow::Error) -> Self {
        AppError::InternalServerError(err.to_string())
    }
}

// Convenience constructors
impl AppError {
    pub fn user_not_found(id: impl Into<String>) -> Self {
        AppError::UserNotFound(id.into())
    }

    pub fn character_not_found(id: impl Into<String>) -> Self {
        AppError::CharacterNotFound(id.into())
    }

    pub fn forbidden() -> Self {
        AppError::Forbidden
    }

    pub fn validation_error(message: impl Into<String>) -> Self {
        AppError::ValidationError(message.into())
    }

    pub fn authentication_error(message: impl Into<String>) -> Self {
        AppError::AuthenticationError(message.into())
    }

    pub fn internal_error(message: impl Into<String>) -> Self {
        AppError::InternalServerError(message.into())
    }
}
