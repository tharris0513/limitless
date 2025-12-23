-- Update character stats from KoL stats to custom stats
-- Replace muscle, mysticism, moxie with might, defense, magic, resistance, agility

-- Add new columns
ALTER TABLE character_stats ADD COLUMN might INTEGER NOT NULL DEFAULT 10;
ALTER TABLE character_stats ADD COLUMN defense INTEGER NOT NULL DEFAULT 10;
ALTER TABLE character_stats ADD COLUMN magic INTEGER NOT NULL DEFAULT 10;
ALTER TABLE character_stats ADD COLUMN resistance INTEGER NOT NULL DEFAULT 10;
ALTER TABLE character_stats ADD COLUMN agility INTEGER NOT NULL DEFAULT 10;

-- Copy existing data (map old stats to new ones for continuity)
-- muscle -> might, mysticism -> magic, moxie -> agility
-- Set defense and resistance to reasonable defaults based on level
UPDATE character_stats SET 
    might = muscle,
    defense = muscle / 2 + 5,
    magic = mysticism,
    resistance = mysticism / 2 + 5,
    agility = moxie;

-- Remove old columns
ALTER TABLE character_stats DROP COLUMN muscle;
ALTER TABLE character_stats DROP COLUMN mysticism;
ALTER TABLE character_stats DROP COLUMN moxie;
