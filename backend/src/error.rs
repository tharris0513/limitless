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

    // Authentication errors
    AuthenticationError(String),
    Forbidden,
    InvalidToken,

    // Validation errors
    ValidationError(String),

    // Business logic errors
    CharacterNotFound(String),

    // Maintenance
    MaintenanceMode,

    // Generic errors
    InternalServerError(String),
    BadRequest(String),
}

impl AppError {
    fn status_code(&self) -> StatusCode {
        match self {
            AppError::DatabaseError(_) => StatusCode::INTERNAL_SERVER_ERROR,

            AppError::AuthenticationError(_) => StatusCode::UNAUTHORIZED,
            AppError::Forbidden => StatusCode::FORBIDDEN,
            AppError::InvalidToken => StatusCode::UNAUTHORIZED,

            AppError::ValidationError(_) => StatusCode::BAD_REQUEST,

            AppError::CharacterNotFound(_) => StatusCode::NOT_FOUND,

            AppError::MaintenanceMode => StatusCode::SERVICE_UNAVAILABLE,

            AppError::InternalServerError(_) => StatusCode::INTERNAL_SERVER_ERROR,
            AppError::BadRequest(_) => StatusCode::BAD_REQUEST,
        }
    }

    fn error_type(&self) -> &'static str {
        match self {
            AppError::DatabaseError(_) => "DATABASE_ERROR",
            AppError::AuthenticationError(_) => "AUTHENTICATION_ERROR",
            AppError::Forbidden | AppError::InvalidToken => "FORBIDDEN",
            AppError::ValidationError(_) => "VALIDATION_ERROR",
            AppError::CharacterNotFound(_) => "RESOURCE_NOT_FOUND",
            AppError::MaintenanceMode => "MAINTENANCE_MODE",
            AppError::InternalServerError(_) => "INTERNAL_SERVER_ERROR",
            AppError::BadRequest(_) => "BAD_REQUEST",
        }
    }

    fn message(&self) -> String {
        match self {
            AppError::DatabaseError(_) => "Database operation failed".to_string(),
            AppError::AuthenticationError(msg) => msg.clone(),
            AppError::Forbidden => "Access to this resource is forbidden".to_string(),
            AppError::InvalidToken => "Invalid authentication token".to_string(),

            AppError::ValidationError(msg) => msg.clone(),
            AppError::CharacterNotFound(id) => format!("Character with ID '{}' not found", id),
            AppError::MaintenanceMode => {
                "The server is currently undergoing maintenance. Please try again later."
                    .to_string()
            }
            AppError::InternalServerError(msg) => format!("Internal server error: {}", msg),
            AppError::BadRequest(msg) => msg.clone(),
        }
    }

    fn details(&self) -> Option<HashMap<String, serde_json::Value>> {
        match self {
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
}
