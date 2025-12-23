use crate::models::{Character, CharacterStats, User};
use anyhow::Result;
use chrono::Utc;
use sqlx::SqlitePool;
use uuid::Uuid;

pub struct UserRepository {
    pub pool: SqlitePool,
}

impl UserRepository {
    pub fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }

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
        let row = sqlx::query!(
            "SELECT id, discord_id, discord_name, admin, created_at FROM users WHERE discord_id = ?",
            discord_id
        )
        .fetch_one(&self.pool)
        .await?;

        Ok(User {
            id: row.id.unwrap_or_default(),
            discord_id: row.discord_id,
            discord_name: row.discord_name,
            admin: row.admin,
            created_at: row.created_at,
        })
    }

    pub async fn find_by_id(&self, id: &str) -> Result<User> {
        let row = sqlx::query!(
            "SELECT id, discord_id, discord_name, admin, created_at FROM users WHERE id = ?",
            id
        )
        .fetch_one(&self.pool)
        .await?;

        Ok(User {
            id: row.id.unwrap_or_default(),
            discord_id: row.discord_id,
            discord_name: row.discord_name,
            admin: row.admin,
            created_at: row.created_at,
        })
    }

    // Admin only - get all users
    pub async fn get_all_users(&self) -> Result<Vec<User>> {
        let rows = sqlx::query!(
            "SELECT id, discord_id, discord_name, admin, created_at FROM users ORDER BY created_at DESC"
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(rows
            .into_iter()
            .map(|row| User {
                id: row.id.unwrap_or_default(),
                discord_id: row.discord_id,
                discord_name: row.discord_name,
                admin: row.admin,
                created_at: row.created_at,
            })
            .collect())
    }

    async fn create_user(&self, discord_id: &str, discord_name: &str) -> Result<User> {
        let id = Uuid::new_v4().to_string();
        let created_at = Utc::now().to_rfc3339();

        sqlx::query!(
            "INSERT INTO users (id, discord_id, discord_name, admin, created_at) VALUES (?, ?, ?, ?, ?)",
            id,
            discord_id,
            discord_name,
            false,
            created_at
        )
        .execute(&self.pool)
        .await?;

        Ok(User {
            id,
            discord_id: discord_id.to_string(),
            discord_name: discord_name.to_string(),
            admin: false,
            created_at,
        })
    }

    pub async fn get_user_characters(&self, user_id: &str) -> Result<Vec<Character>> {
        let mut characters = Vec::new();

        let character_rows = sqlx::query!(
            "SELECT id, user_id, name, level, health, max_health, mana, max_mana, experience, experience_to_next, location, created_at, last_played FROM characters WHERE user_id = ? ORDER BY last_played DESC",
            user_id
        )
        .fetch_all(&self.pool)
        .await?;

        for row in character_rows {
            // Get character stats
            let id = row.id.as_ref().unwrap();
            let stats = self.get_character_stats(id).await?;

            characters.push(Character {
                id: id.to_string(),
                user_id: row.user_id,
                name: row.name,
                level: row.level,
                health: row.health,
                max_health: row.max_health,
                mana: row.mana,
                max_mana: row.max_mana,
                experience: row.experience,
                experience_to_next: row.experience_to_next,
                stats,
                location: row.location,
                created_at: row.created_at,
                last_played: row.last_played,
            });
        }

        Ok(characters)
    }

    pub async fn create_character(&self, user_id: &str, name: &str) -> Result<Character> {
        let character_id = Uuid::new_v4().to_string();
        let created_at = Utc::now().to_rfc3339();
        let last_played = created_at.clone();

        // Create character
        sqlx::query!(
            "INSERT INTO characters (id, user_id, name, level, health, max_health, mana, max_mana, experience, experience_to_next, location, created_at, last_played) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)",
            character_id,
            user_id,
            name,
            1,      // level
            100,    // health
            100,    // max_health
            50,     // mana
            50,     // max_mana
            0,      // experience
            100,    // experience_to_next
            "starting_area", // location
            created_at,
            last_played
        )
        .execute(&self.pool)
        .await?;

        // Create character stats
        let default_stats = CharacterStats {
            might: 10,
            defense: 10,
            magic: 10,
            resistance: 10,
            agility: 10,
            adventures: 5,
            max_adventures: 5,
        };

        self.create_character_stats(&character_id, &default_stats)
            .await?;

        // Create empty equipment record
        sqlx::query!(
            "INSERT INTO character_equipment (character_id, weapon, armor, accessory) VALUES (?, ?, ?, ?)",
            character_id,
            Option::<String>::None,
            Option::<String>::None,
            Option::<String>::None
        )
        .execute(&self.pool)
        .await?;

        Ok(Character {
            id: character_id,
            user_id: user_id.to_string(),
            name: name.to_string(),
            level: 1,
            health: 100,
            max_health: 100,
            mana: 50,
            max_mana: 50,
            experience: 0,
            experience_to_next: 100,
            stats: default_stats,
            location: "starting_area".to_string(),
            created_at,
            last_played,
        })
    }

    pub async fn update_character_last_played(&self, character_id: &str) -> Result<()> {
        let last_played = Utc::now().to_rfc3339();

        sqlx::query!(
            "UPDATE characters SET last_played = ? WHERE id = ?",
            last_played,
            character_id
        )
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    async fn get_character_stats(&self, character_id: &str) -> Result<CharacterStats> {
        let stats = sqlx::query!(
            "SELECT might, defense, magic, resistance, agility, adventures, max_adventures FROM character_stats WHERE character_id = ?",
            character_id
        )
        .fetch_one(&self.pool)
        .await?;

        Ok(CharacterStats {
            might: stats.might,
            defense: stats.defense,
            magic: stats.magic,
            resistance: stats.resistance,
            agility: stats.agility,
            adventures: stats.adventures,
            max_adventures: stats.max_adventures,
        })
    }

    async fn create_character_stats(
        &self,
        character_id: &str,
        stats: &CharacterStats,
    ) -> Result<()> {
        sqlx::query!(
            "INSERT INTO character_stats (character_id, might, defense, magic, resistance, agility, adventures, max_adventures) VALUES (?, ?, ?, ?, ?, ?, ?, ?)",
            character_id,
            stats.might,
            stats.defense,
            stats.magic,
            stats.resistance,
            stats.agility,
            stats.adventures,
            stats.max_adventures
        )
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    // Health check - verify database connectivity
    pub async fn health_check(&self) -> Result<()> {
        sqlx::query!("SELECT 1 as health_check")
            .fetch_one(&self.pool)
            .await?;
        Ok(())
    }
}
