use crate::repository::UserRepository;
use std::sync::Arc;

/// Perform the midnight rollover
/// 1. Enable maintenance mode
/// 2. Invalidate all JWT tokens
/// 3. Add 50 adventures to all characters (max 200) and restore health/mana to max
/// 4. Disable maintenance mode
pub async fn perform_rollover(repo: Arc<UserRepository>) {
    tracing::info!("🌙 Starting midnight rollover...");

    // Step 1: Enable maintenance mode
    if let Err(e) = repo.set_maintenance_mode(true).await {
        tracing::error!("Failed to enable maintenance mode: {}", e);
        return;
    }
    tracing::info!("✓ Maintenance mode enabled");

    // Step 2: Invalidate all JWT tokens by incrementing generation
    match repo.increment_jwt_generation().await {
        Ok(new_gen) => {
            tracing::info!(
                "✓ JWT generation incremented to {}, all tokens invalidated",
                new_gen
            );
        }
        Err(e) => {
            tracing::error!("Failed to increment JWT generation: {}", e);
            // Try to disable maintenance mode before returning
            let _ = repo.set_maintenance_mode(false).await;
            return;
        }
    }

    // Step 3: Add 50 adventures to all characters (max 200) and restore health/mana
    match repo.get_all_characters_for_rollover().await {
        Ok(characters) => {
            tracing::info!("Found {} characters to update", characters.len());
            let mut success_count = 0;
            let mut error_count = 0;

            for (user_id, character_id) in characters {
                match repo
                    .add_adventures_to_character(&user_id, &character_id, 50, 200)
                    .await
                {
                    Ok(_) => success_count += 1,
                    Err(e) => {
                        tracing::error!(
                            "Failed to update character {} for user {}: {:?}",
                            character_id,
                            user_id,
                            e
                        );
                        error_count += 1;
                    }
                }
            }

            tracing::info!(
                "✓ Rollover complete (adventures, health, mana): {} success, {} errors",
                success_count,
                error_count
            );
        }
        Err(e) => {
            tracing::error!("Failed to get characters for rollover: {}", e);
            // Try to disable maintenance mode before returning
            let _ = repo.set_maintenance_mode(false).await;
            return;
        }
    }

    // Step 4: Disable maintenance mode
    if let Err(e) = repo.set_maintenance_mode(false).await {
        tracing::error!("Failed to disable maintenance mode: {}", e);
        return;
    }
    tracing::info!("✓ Maintenance mode disabled");

    tracing::info!("🌙 Midnight rollover complete!");
}
