use crate::models::{Claims, User};
use crate::repository::UserRepository;
use anyhow::Result;
use chrono::{Duration, Utc};
use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation};
use std::env;
use std::sync::Arc;

pub struct JwtService;

impl JwtService {
    fn get_jwt_secret() -> String {
        env::var("JWT_SECRET")
            .unwrap_or_else(|_| "your-super-secret-jwt-key-change-in-production".to_string())
    }

    pub async fn generate_token_with_repo(
        user: &User,
        repo: &Arc<UserRepository>,
    ) -> Result<String> {
        let expiration = Utc::now()
            .checked_add_signed(Duration::hours(24))
            .expect("valid timestamp")
            .timestamp() as usize;

        // Get current generation from database
        let generation = repo.get_jwt_generation().await.unwrap_or(0);

        let claims = Claims {
            sub: user.id.clone(),
            discord_id: user.discord_id.clone(),
            admin: user.admin,
            exp: expiration,
            gen: generation,
        };

        let token = encode(
            &Header::default(),
            &claims,
            &EncodingKey::from_secret(Self::get_jwt_secret().as_ref()),
        )?;

        Ok(token)
    }

    // Legacy method for backward compatibility (uses env var)
    pub fn generate_token(user: &User) -> Result<String> {
        let expiration = Utc::now()
            .checked_add_signed(Duration::hours(24))
            .expect("valid timestamp")
            .timestamp() as usize;

        // Get current generation from environment or default to 0
        let generation = Self::get_current_generation();

        let claims = Claims {
            sub: user.id.clone(),
            discord_id: user.discord_id.clone(),
            admin: user.admin,
            exp: expiration,
            gen: generation,
        };

        let token = encode(
            &Header::default(),
            &claims,
            &EncodingKey::from_secret(Self::get_jwt_secret().as_ref()),
        )?;

        Ok(token)
    }

    pub async fn verify_token_with_repo(token: &str, repo: &Arc<UserRepository>) -> Result<Claims> {
        let token_data = decode::<Claims>(
            token,
            &DecodingKey::from_secret(Self::get_jwt_secret().as_ref()),
            &Validation::default(),
        )?;

        // Verify generation matches current generation from database
        let current_gen = repo.get_jwt_generation().await.unwrap_or(0);
        if token_data.claims.gen < current_gen {
            return Err(anyhow::anyhow!("Token has been invalidated"));
        }

        Ok(token_data.claims)
    }

    // Legacy method for backward compatibility
    pub fn verify_token(token: &str) -> Result<Claims> {
        let token_data = decode::<Claims>(
            token,
            &DecodingKey::from_secret(Self::get_jwt_secret().as_ref()),
            &Validation::default(),
        )?;

        // Verify generation matches current generation
        let current_gen = Self::get_current_generation();
        if token_data.claims.gen < current_gen {
            return Err(anyhow::anyhow!("Token has been invalidated"));
        }

        Ok(token_data.claims)
    }

    pub fn get_current_generation() -> usize {
        env::var("JWT_GENERATION")
            .ok()
            .and_then(|v| v.parse().ok())
            .unwrap_or(0)
    }

    pub fn increment_generation() -> Result<usize> {
        let current = Self::get_current_generation();
        let new_gen = current + 1;
        // Note: In production, this should be persisted to a database or shared cache
        // For now, we'll return the new generation but the caller must persist it
        Ok(new_gen)
    }

    pub fn generate_token_for_claims(claims: &Claims) -> Result<String> {
        let token = encode(
            &Header::default(),
            claims,
            &EncodingKey::from_secret(Self::get_jwt_secret().as_ref()),
        )?;
        Ok(token)
    }

    pub fn extract_token_from_auth_header(auth_header: &str) -> Option<&str> {
        if auth_header.starts_with("Bearer ") {
            Some(&auth_header[7..])
        } else {
            None
        }
    }
}
