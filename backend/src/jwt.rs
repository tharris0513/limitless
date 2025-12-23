use crate::models::{Claims, User};
use anyhow::Result;
use chrono::{Duration, Utc};
use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation};
use std::env;

pub struct JwtService;

impl JwtService {
    fn get_jwt_secret() -> String {
        env::var("JWT_SECRET").unwrap_or_else(|_| "your-super-secret-jwt-key-change-in-production".to_string())
    }

    pub fn generate_token(user: &User) -> Result<String> {
        let expiration = Utc::now()
            .checked_add_signed(Duration::hours(24))
            .expect("valid timestamp")
            .timestamp() as usize;

        let claims = Claims {
            sub: user.id.clone(),
            discord_id: user.discord_id.clone(),
            admin: user.admin,
            exp: expiration,
        };

        let token = encode(
            &Header::default(),
            &claims,
            &EncodingKey::from_secret(Self::get_jwt_secret().as_ref()),
        )?;

        Ok(token)
    }

    pub fn verify_token(token: &str) -> Result<Claims> {
        let token_data = decode::<Claims>(
            token,
            &DecodingKey::from_secret(Self::get_jwt_secret().as_ref()),
            &Validation::default(),
        )?;

        Ok(token_data.claims)
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