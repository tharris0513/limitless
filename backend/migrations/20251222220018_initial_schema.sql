-- Initial database schema for Limitless game

-- Users table - represents Discord users
CREATE TABLE users (
    id TEXT PRIMARY KEY, -- UUID as string
    discord_id TEXT NOT NULL UNIQUE, -- Discord user ID from OAuth
    discord_name TEXT NOT NULL, -- Discord username
    admin BOOLEAN NOT NULL DEFAULT FALSE,
    created_at TEXT NOT NULL -- ISO 8601 timestamp
);

-- Characters table - game characters that belong to users
CREATE TABLE characters (
    id TEXT PRIMARY KEY, -- UUID as string
    user_id TEXT NOT NULL,
    name TEXT NOT NULL,
    level INTEGER NOT NULL DEFAULT 1,
    health INTEGER NOT NULL DEFAULT 100,
    max_health INTEGER NOT NULL DEFAULT 100,
    mana INTEGER NOT NULL DEFAULT 50,
    max_mana INTEGER NOT NULL DEFAULT 50,
    experience INTEGER NOT NULL DEFAULT 0,
    experience_to_next INTEGER NOT NULL DEFAULT 100,
    location TEXT NOT NULL DEFAULT 'starting_area',
    created_at TEXT NOT NULL, -- ISO 8601 timestamp
    last_played TEXT NOT NULL, -- ISO 8601 timestamp
    FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE
);

-- Character stats table
CREATE TABLE character_stats (
    character_id TEXT PRIMARY KEY,
    muscle INTEGER NOT NULL DEFAULT 10,
    mysticism INTEGER NOT NULL DEFAULT 10,
    moxie INTEGER NOT NULL DEFAULT 10,
    adventures INTEGER NOT NULL DEFAULT 5,
    max_adventures INTEGER NOT NULL DEFAULT 5,
    FOREIGN KEY (character_id) REFERENCES characters(id) ON DELETE CASCADE
);

-- Character inventory table
CREATE TABLE character_inventory (
    id TEXT PRIMARY KEY, -- UUID as string
    character_id TEXT NOT NULL,
    item_id TEXT NOT NULL,
    quantity INTEGER NOT NULL DEFAULT 1,
    FOREIGN KEY (character_id) REFERENCES characters(id) ON DELETE CASCADE
);

-- Character equipment table
CREATE TABLE character_equipment (
    character_id TEXT PRIMARY KEY,
    weapon TEXT,
    armor TEXT,
    accessory TEXT,
    FOREIGN KEY (character_id) REFERENCES characters(id) ON DELETE CASCADE
);

-- Create indexes for better performance
CREATE INDEX idx_characters_user_id ON characters(user_id);
CREATE INDEX idx_characters_last_played ON characters(last_played);
CREATE INDEX idx_inventory_character_id ON character_inventory(character_id);
