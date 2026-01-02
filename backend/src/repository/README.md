# Repository Module Structure

This directory contains the refactored repository implementation, split into domain-specific modules for better maintainability and code organization.

## File Structure

```
repository/
├── mod.rs          # Main module file - defines UserRepository struct
├── user.rs         # User CRUD operations
├── character.rs    # Character CRUD operations and game state management
├── class.rs        # Class CRUD operations and class-ability associations
├── ability.rs      # Ability CRUD operations and character ability unlocking
└── config.rs       # Game configuration import/export and health checks
```

## Module Responsibilities

### `user.rs` (9 methods)

User-related database operations:

- `find_or_create_by_discord_id` - Find or create user by Discord ID
- `find_by_discord_id` - Find user by Discord ID
- `find_by_id` - Find user by internal ID
- `get_all_users` - Admin: Get all users (uses entity-type-index)
- `create_user` - Create new user
- `parse_user` - Parse DynamoDB item to User struct
- `update_user` - Update user profile (username, DOB)
- `delete_user` - Delete user and all their characters
- `update_user_ban_status` - Update user ban status

### `character.rs` (9 methods)

Character-related database operations:

- `get_user_characters` - Get all characters for a user
- `create_character` - Create new character with initial stats and level 1 abilities
- `update_character_last_played` - Update last played timestamp
- `update_character_game_state` - Save/clear game state (combat, choice, idle)
- `parse_character` - Parse DynamoDB item to Character struct
- `get_all_characters` - Admin: Get all characters from all users
- `update_character_name` - Admin: Update character name
- `update_character_adventures` - Admin: Update character adventure count
- `delete_character` - Admin: Delete character

### `class.rs` (8 methods)

Class-related database operations:

- `get_class` - Get class by ID
- `get_all_classes` - Get all available classes
- `parse_class` - Parse DynamoDB item to Class struct
- `create_class` - Create new class
- `update_class` - Update existing class
- `delete_class` - Delete class and all its ability associations
- `add_class_ability` - Add ability to class with unlock level
- `remove_class_ability` - Remove ability from class

### `ability.rs` (11 methods + helpers)

Ability-related database operations:

- `create_ability` - Create new ability
- `get_ability` - Get ability by ID
- `get_all_abilities` - Get all abilities
- `update_ability` - Update existing ability
- `delete_ability` - Delete ability
- `parse_ability` - Parse DynamoDB item to Ability struct
- `get_class_abilities` - Get abilities for a class
- `get_character_abilities` - Get unlocked abilities for a character
- `unlock_character_abilities` - Unlock abilities when character levels up
- `parse_ability_from_class_item` - Helper: Parse ability from class association
- `get_unlock_level` - Helper: Extract unlock level from item
- `parse_character_ability` - Helper: Parse character-ability association
- `get_class_abilities_full` - Helper: Get class abilities with IDs and levels (for export)

### `config.rs` (3 methods)

Game configuration and system operations:

- `export_game_config` - Export all classes and abilities as JSON
- `import_game_config` - Import game configuration from JSON
- `health_check` - Verify DynamoDB connectivity

## Design Pattern

Each module file contains a **partial implementation** of `UserRepository` using Rust's support for splitting impl blocks across modules:

```rust
// mod.rs - Defines the struct
pub struct UserRepository {
    pub client: DynamoDbClient,
    pub table_name: String,
}

// user.rs - Adds user methods
use super::UserRepository;
impl UserRepository {
    pub async fn find_by_id(&self, id: &str) -> Result<User> { ... }
}

// character.rs - Adds character methods
use super::UserRepository;
impl UserRepository {
    pub async fn create_character(&self, ...) -> Result<Character> { ... }
}
```

This approach allows:

- ✅ Clear separation of concerns by domain
- ✅ Easier navigation and maintenance (each file is ~200-400 lines)
- ✅ Parallel development without merge conflicts
- ✅ All methods still available on the same `UserRepository` instance
- ✅ Maintains the existing public API - no changes required in handlers

## Helper Methods

Methods marked `pub(crate)` are internal helpers used across repository modules:

- `parse_user`, `parse_character`, `parse_class`, `parse_ability` - DynamoDB parsing
- `unlock_character_abilities` - Called during character creation
- `get_class_abilities_full` - Used by export functionality

## Usage

No changes required in application code - the repository is used exactly the same way:

```rust
let repo = Arc::new(repository::UserRepository::new(client, table_name));

// All methods are available as before
let users = repo.get_all_users().await?;
let character = repo.create_character(user_id, name, class_id).await?;
let config = repo.export_game_config().await?;
```

## Migration Notes

Previously: Single `repository.rs` file with 1399 lines

Now: 5 focused module files with clear responsibilities:

- `user.rs`: ~280 lines
- `character.rs`: ~350 lines
- `class.rs`: ~220 lines
- `ability.rs`: ~320 lines
- `config.rs`: ~80 lines
- `mod.rs`: ~15 lines

Total: ~1265 lines (more readable structure, same functionality)
