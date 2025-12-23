import {
  type Player,
  type Location,
  type Adventure,
  ItemType,
  Rarity,
} from '../types/game';

// Mock player data for development
export const mockPlayer: Player = {
  id: '1',
  username: 'TestAdventurer',
  level: 5,
  health: 80,
  maxHealth: 100,
  mana: 45,
  maxMana: 60,
  experience: 1250,
  experienceToNext: 1500,
  stats: {
    might: 12,
    defense: 8,
    magic: 8,
    resistance: 6,
    agility: 15,
    adventures: 8,
    maxAdventures: 10,
  },
  inventory: [
    {
      id: 'sword1',
      name: 'Rusty Sword',
      description: 'An old sword that has seen better days.',
      quantity: 1,
      type: ItemType.WEAPON,
      rarity: Rarity.COMMON,
    },
    {
      id: 'potion1',
      name: 'Health Potion',
      description: 'Restores 50 health points.',
      quantity: 3,
      type: ItemType.CONSUMABLE,
      rarity: Rarity.COMMON,
    },
  ],
  equipment: {
    weapon: {
      id: 'sword1',
      name: 'Rusty Sword',
      description: 'An old sword that has seen better days.',
      quantity: 1,
      type: ItemType.WEAPON,
      rarity: Rarity.COMMON,
    },
  },
  location: 'Hometown',
};

export const mockLocations: Location[] = [
  {
    id: 'hometown',
    name: 'Hometown',
    description: 'A peaceful village where your adventure begins.',
    adventures: [
      {
        id: 'training_dummy',
        name: 'Training with the Dummy',
        description: 'Practice your combat skills on the training dummy.',
        difficulty: 1,
        rewards: [{ type: 'experience', amount: 50 }],
      },
      {
        id: 'help_farmer',
        name: 'Help the Local Farmer',
        description: 'A farmer needs help dealing with some pesky goblins.',
        difficulty: 2,
        rewards: [
          { type: 'experience', amount: 100 },
          { type: 'currency', amount: 25 },
        ],
      },
    ],
    shops: [],
  },
  {
    id: 'dark_forest',
    name: 'Dark Forest',
    description: 'A mysterious forest filled with dangerous creatures.',
    adventures: [
      {
        id: 'forest_patrol',
        name: 'Forest Patrol',
        description: 'Patrol the forest paths and clear out any threats.',
        difficulty: 4,
        rewards: [{ type: 'experience', amount: 200 }],
      },
      {
        id: 'ancient_ruins',
        name: 'Explore Ancient Ruins',
        description: 'Investigate the mysterious ruins deep in the forest.',
        difficulty: 6,
        rewards: [{ type: 'experience', amount: 350 }],
      },
    ],
    shops: [],
  },
  {
    id: 'mountain_pass',
    name: 'Mountain Pass',
    description: 'Treacherous mountain paths with hidden treasures.',
    adventures: [
      {
        id: 'mountain_climb',
        name: 'Scale the Peak',
        description: 'Climb to the highest peak and claim its treasure.',
        difficulty: 8,
        rewards: [{ type: 'experience', amount: 500 }],
      },
    ],
    shops: [],
  },
];

// Mock API responses with delays to simulate network calls
export const MockAPI = {
  async login(
    username: string,
    password: string
  ): Promise<{ token: string; player: Player }> {
    await delay(1000);
    if (username === 'demo' && password === 'demo') {
      return {
        token: 'mock-jwt-token',
        player: mockPlayer,
      };
    }
    throw new Error('Invalid credentials');
  },

  async register(
    username: string,
    _: string,
    __: string
  ): Promise<{ token: string; player: Player }> {
    await delay(1500);
    return {
      token: 'mock-jwt-token',
      player: { ...mockPlayer, username },
    };
  },

  async getPlayer(): Promise<Player> {
    await delay(500);
    return mockPlayer;
  },

  async getLocations(): Promise<Location[]> {
    await delay(800);
    return mockLocations;
  },

  async startAdventure(adventureId: string): Promise<Adventure> {
    await delay(1200);
    const location = mockLocations.find(l =>
      l.adventures.some(a => a.id === adventureId)
    );
    const adventure = location?.adventures.find(a => a.id === adventureId);
    if (!adventure) throw new Error('Adventure not found');
    return adventure;
  },

  async completeAdventure(
    _: string
  ): Promise<{ rewards: { type: string; amount: number }[]; player: Player }> {
    await delay(2000); // Simulate adventure time

    // Update mock player with rewards
    const updatedPlayer = {
      ...mockPlayer,
      experience: mockPlayer.experience + 100,
      stats: {
        ...mockPlayer.stats,
        adventures: mockPlayer.stats.adventures - 1,
      },
    };

    return {
      rewards: [{ type: 'experience', amount: 100 }],
      player: updatedPlayer,
    };
  },
};

function delay(ms: number): Promise<void> {
  return new Promise(resolve => setTimeout(resolve, ms));
}
