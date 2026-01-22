// User represents the Discord account holder
export interface User {
  id: string; // User UUID
  discordId: string; // Discord ID
  discordName: string; // Discord username with discriminator
  username?: string; // User's chosen username (optional until account setup complete)
  dateOfBirth?: string; // User's date of birth (optional until account setup complete)
  admin: boolean; // Whether the user is an admin
  banned: boolean; // Whether the user is banned from the game
  createdAt: string; // When user first logged in
}

// Active buff tracking for noncombat abilities
export interface ActiveBuff {
  abilityId: string;
  abilityName: string;
  remainingAdventures: number;
  effects: AbilityEffect[];
}

// Character represents a game character that belongs to a user
export interface Character {
  id: string; // Character UUID
  userId: string; // Foreign key to User
  name: string; // Character name (chosen by player)
  classId: string; // Character's class
  level: number;
  health: number;
  maxHealth: number;
  mana: number;
  maxMana: number;
  experience: number;
  experienceToNext: number;
  might: number; // Physical power
  defense: number; // Physical defense
  magic: number; // Magical power
  resistance: number; // Magical defense
  agility: number; // Speed
  adventures: number;
  inventory: InventoryItem[];
  equipment: Equipment;
  location: string;
  gameState?: string; // JSON string storing current game state
  activeBuffs?: ActiveBuff[]; // Currently active noncombat ability buffs
  createdAt: string;
  lastPlayed: string;
}

// Class represents a character class with base stats
export interface Class {
  id: string;
  name: string;
  description: string;
  startingMight: number;
  startingDefense: number;
  startingMagic: number;
  startingResistance: number;
  startingAgility: number;
  startingHealth: number;
  startingMana: number;
}

// Represents a single effect within an ability
export interface AbilityEffect {
  id: string; // Unique ID for this effect
  effectType: 'active' | 'passive';

  // For active effects
  activeType?: 'damage' | 'heal';
  formula?: string;
  attackDescription?: string;

  // For passive effects
  passiveType?: string; // e.g., "dual_wield", "increased_crit"
  passiveMode?: 'effect' | 'stat_modifier'; // Mode for passive effects
  statModifier?: {
    stat:
      | 'might'
      | 'defense'
      | 'magic'
      | 'resistance'
      | 'agility'
      | 'maxHealth'
      | 'maxMana';
    value: number;
    type: 'flat' | 'percentage' | 'set';
  };
}

// Ability represents a skill or spell that characters can use
export interface Ability {
  id: string;
  name: string;
  description: string;

  // Multi-effect system (new)
  effects?: AbilityEffect[];

  manaCost: number;
  cooldown: number;
  duration?: number; // For noncombat abilities - how many adventures the buff lasts

  // Ability type determines visibility in combat
  abilityType: 'combat' | 'passive' | 'noncombat';

  // Legacy fields (deprecated but kept for backward compatibility)
  damageFormula?: string;
  healFormula?: string;
  effectFormula?: string;
  passiveEffect?: string;
  attackDescription?: string;
}

// Passive effect option
export interface PassiveEffect {
  id: string;
  description: string;
}

// Combat attack result
export interface AttackResult {
  attacks: SingleAttack[];
  totalDamage: number;
  enemyHealth: number;
  enemyAttacks: SingleAttack[];
  playerHealth: number;
  playerMana?: number;
  victory: boolean;
  defeat: boolean;
  experienceGained?: number;
  victoryMessage?: string;
  defeatMessage?: string;
  levelUp?: LevelUpInfo;
  gameState?: any; // Updated game state from backend (null on victory)
}

export interface LevelUpInfo {
  newLevel: number;
  statIncreases: StatIncreases;
  abilitiesLearned: AbilityLearned[];
}

export interface AbilityLearned {
  id: string;
  name: string;
  description: string;
}

export interface StatIncreases {
  might: number;
  defense: number;
  magic: number;
  resistance: number;
  agility: number;
  maxHealth: number;
  maxMana: number;
}

export interface SingleAttack {
  damage: number;
  description: string;
  isDualWield: boolean;
}

// CharacterAbility tracks which abilities a character has unlocked
export interface CharacterAbility {
  characterId: string;
  abilityId: string;
  unlockedAtLevel: number;
  ability?: Ability; // Joined ability data
}

// Auth response from backend
export interface AuthResponse {
  token: string;
  user: User;
  characters: Character[];
}

export interface InventoryItem {
  id: string;
  name: string;
  description: string;
  quantity: number;
  type: ItemType;
  rarity: Rarity;
}

export interface Equipment {
  weapon?: InventoryItem;
  armor?: InventoryItem;
  accessory?: InventoryItem;
}

// Item type constants and union type
export const ItemType = {
  WEAPON: 'weapon',
  ARMOR: 'armor',
  ACCESSORY: 'accessory',
  CONSUMABLE: 'consumable',
  QUEST: 'quest',
  MISC: 'misc',
} as const;

export type ItemType = (typeof ItemType)[keyof typeof ItemType];

// Rarity constants and union type
export const Rarity = {
  COMMON: 'common',
  UNCOMMON: 'uncommon',
  RARE: 'rare',
  EPIC: 'epic',
  LEGENDARY: 'legendary',
} as const;

export type Rarity = (typeof Rarity)[keyof typeof Rarity];

// Location represents a game location/area
export interface Location {
  id: string;
  name: string;
  description: string;
  minLevel: number;
  maxLevel: number;
  tier: number;
  enabled: boolean;
  createdAt: string;
}

// Creature represents an enemy or NPC
export interface Creature {
  id: string;
  name: string;
  introductionText: string;
  level: number;
  health: number;
  might: number;
  defense: number;
  magic: number;
  resistance: number;
  agility: number;
  experienceReward: number;
  creatureType: string; // 'beast', 'undead', 'humanoid', 'elemental', 'dragon', 'demon', 'horror'
  attackDescription?: string; // Template for attack descriptions, e.g., "The ${name} attacks you for ${damage} damage!"
  createdAt: string;
}

// Adventure represents a noncombat encounter/event
export interface Adventure {
  id: string;
  name: string;
  description: string;
  requiredLevel: number;
  adventureType: string; // 'puzzle', 'dialogue', 'exploration', etc.
  experienceReward: number;
  createdAt: string;
}

// LocationCreature associates creatures with locations
export interface LocationCreature {
  locationId: string;
  creatureId: string;
  spawnRate: number; // 1-100
}

// LocationAdventure associates adventures with locations
export interface LocationAdventure {
  locationId: string;
  adventureId: string;
  spawnRate: number; // 1-100
}

// Legacy types for backward compatibility
export interface LegacyLocation {
  id: string;
  name: string;
  description: string;
  adventures: LegacyAdventure[];
  shops: Shop[];
}

export interface LegacyAdventure {
  id: string;
  name: string;
  description: string;
  difficulty: number;
  rewards: Reward[];
}

export interface Reward {
  type: 'experience' | 'item' | 'currency';
  amount?: number;
  item?: InventoryItem;
}

export interface Shop {
  id: string;
  name: string;
  items: ShopItem[];
}

export interface ShopItem {
  item: InventoryItem;
  price: number;
  currency: 'gold' | 'tokens' | 'gems';
}
