// User represents the Discord account holder
export interface User {
  id: string; // User UUID
  discordId: string; // Discord ID
  discordName: string; // Discord username with discriminator
  username?: string; // User's chosen username (optional until account setup complete)
  dateOfBirth?: string; // User's date of birth (optional until account setup complete)
  admin: boolean; // Whether the user is an admin
  createdAt: string; // When user first logged in
}

// Character represents a game character that belongs to a user
export interface Character {
  id: string; // Character UUID
  userId: string; // Foreign key to User
  name: string; // Character name (chosen by player)
  level: number;
  health: number;
  maxHealth: number;
  mana: number;
  maxMana: number;
  experience: number;
  experienceToNext: number;
  stats: CharacterStats;
  inventory: InventoryItem[];
  equipment: Equipment;
  location: string;
  createdAt: string;
  lastPlayed: string;
}

// Auth response from backend
export interface AuthResponse {
  token: string;
  user: User;
  characters: Character[];
}

// Legacy Player type - alias for Character for backward compatibility
export type Player = Character;

export interface CharacterStats {
  might: number; // Physical power
  defense: number; // Physical defense
  magic: number; // Magical power
  resistance: number; // Magical defense
  agility: number; // Speed
  adventures: number;
  maxAdventures: number;
}

// Legacy PlayerStats type - alias for CharacterStats for backward compatibility
export type PlayerStats = CharacterStats;

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

export interface Location {
  id: string;
  name: string;
  description: string;
  adventures: Adventure[];
  shops: Shop[];
}

export interface Adventure {
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
