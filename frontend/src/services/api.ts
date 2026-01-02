import axios from 'axios';
import type {
  Player,
  User,
  Character,
  CharacterAbility,
  Class,
  Ability,
  Location,
  Adventure,
  Shop,
  ItemType,
} from '../types/game';

// Configure axios instance
const api = axios.create({
  baseURL: import.meta.env.VITE_API_URL || 'http://localhost:8080/api',
  timeout: 10000,
  withCredentials: true, // Enable credentials to send cookies
});

// Add auth token to requests if available
api.interceptors.request.use(config => {
  const token = localStorage.getItem('authToken');
  if (token) {
    config.headers.Authorization = `Bearer ${token}`;
  }
  return config;
});

// Add response interceptor for better error handling
api.interceptors.response.use(
  response => response,
  error => {
    console.error('API Error:', error);
    return Promise.reject(error);
  }
);

// Helper functions to transform between backend snake_case and frontend camelCase
function transformUserFromBackend(backendUser: any): User {
  return {
    id: backendUser.id,
    discordId: backendUser.discord_id, // Transform snake_case to camelCase
    discordName: backendUser.discord_name, // Transform snake_case to camelCase
    username: backendUser.username,
    dateOfBirth: backendUser.date_of_birth, // Transform snake_case to camelCase
    admin: backendUser.admin,
    banned: backendUser.banned || false, // Transform banned field
    createdAt: backendUser.created_at, // Transform snake_case to camelCase
  };
}

function transformUserToBackend(frontendUser: any): any {
  return {
    username: frontendUser.username,
    dateOfBirth: frontendUser.dateOfBirth, // Keep camelCase, backend expects it in the update payload
  };
}

export class GameAPI {
  // Check if user is authenticated via cookie
  static async getAuthFromCookie(): Promise<{ token: string; user: User }> {
    const response = await api.get('/auth/cookie');
    return response.data;
  }

  // Discord OAuth2 endpoints
  static async getDiscordAuthUrl(): Promise<{
    auth_url: string;
    state: string;
  }> {
    const response = await api.get('/auth/discord');
    return response.data;
  }

  static async handleDiscordCallback(
    code: string,
    state?: string
  ): Promise<{ token: string; player: Player }> {
    const params = new URLSearchParams({ code });
    if (state) {
      params.append('state', state);
    }

    const response = await api.get(
      `/auth/discord/callback?${params.toString()}`
    );
    return response.data;
  }

  // Legacy auth endpoints (kept for fallback)
  static async login(
    username: string,
    password: string
  ): Promise<{ token: string; player: Player }> {
    const response = await api.post('/auth/login', { username, password });
    return response.data;
  }

  static async register(
    username: string,
    email: string,
    password: string
  ): Promise<{ token: string; player: Player }> {
    const response = await api.post('/auth/register', {
      username,
      email,
      password,
    });
    return response.data;
  }

  static async logout(): Promise<void> {
    await api.post('/auth/logout');
    localStorage.removeItem('authToken');
  }

  // User endpoints
  static async getUser(): Promise<User> {
    const response = await api.get('/user');
    return transformUserFromBackend(response.data);
  }

  static async updateUser(updates: {
    username?: string;
    dateOfBirth?: string;
  }): Promise<User> {
    const response = await api.patch('/user', transformUserToBackend(updates));
    return transformUserFromBackend(response.data);
  }

  // Admin endpoints
  static async getAllUsers(): Promise<User[]> {
    const response = await api.get('/admin/users');
    return response.data.map((user: any) => transformUserFromBackend(user));
  }

  // Public username check (no auth required)
  static async checkUsernameAvailable(username: string): Promise<boolean> {
    const response = await api.post('/user/check-username', { username });
    return response.data.available;
  }

  // Character endpoints
  static async getUserCharacters(): Promise<Character[]> {
    const response = await api.get('/characters');
    return response.data;
  }

  static async createCharacter(characterData: {
    userId: string;
    name: string;
    classId: string;
  }): Promise<Character> {
    const response = await api.post('/characters', characterData);
    return response.data;
  }

  static async updateCharacter(
    characterId: string,
    updates: Partial<Character>
  ): Promise<Character> {
    const response = await api.patch(`/characters/${characterId}`, updates);
    return response.data;
  }

  static async updateCharacterLastPlayed(characterId: string): Promise<void> {
    await api.patch(`/characters/${characterId}/last-played`);
  }

  static async getCharacterAbilities(
    characterId: string
  ): Promise<CharacterAbility[]> {
    const response = await api.get(`/characters/${characterId}/abilities`);
    return response.data;
  }

  // Class endpoints
  static async getClasses(): Promise<Class[]> {
    const response = await api.get('/classes');
    return response.data;
  }

  static async getClassAbilities(
    classId: string
  ): Promise<Array<{ ability: Ability; unlockLevel: number }>> {
    const response = await api.get(`/classes/${classId}/abilities`);
    // Backend returns array of tuples [Ability, unlockLevel], transform to objects
    return response.data.map((item: [Ability, number]) => ({
      ability: item[0],
      unlockLevel: item[1],
    }));
  }

  // Legacy Player endpoints (for backward compatibility)
  static async getPlayer(): Promise<Player> {
    const response = await api.get('/player');
    return response.data;
  }

  static async updatePlayer(updates: Partial<Player>): Promise<Player> {
    const response = await api.patch('/player', updates);
    return response.data;
  }

  // Game world endpoints
  static async getLocations(): Promise<Location[]> {
    const response = await api.get('/locations');
    return response.data;
  }

  static async getLocation(locationId: string): Promise<Location> {
    const response = await api.get(`/locations/${locationId}`);
    return response.data;
  }

  // Adventure endpoints
  static async startAdventure(adventureId: string): Promise<Adventure> {
    const response = await api.post(`/adventures/${adventureId}/start`);
    return response.data;
  }

  static async completeAdventure(
    adventureId: string,
    choices?: Record<string, ItemType[]>
  ): Promise<{ rewards: ItemType[]; player: Player }> {
    const response = await api.post(`/adventures/${adventureId}/complete`, {
      choices,
    });
    return response.data;
  }

  // Shop endpoints
  static async getShop(shopId: string): Promise<Shop> {
    const response = await api.get(`/shops/${shopId}`);
    return response.data;
  }

  static async purchaseItem(
    shopId: string,
    itemId: string,
    quantity: number = 1
  ): Promise<Player> {
    const response = await api.post(`/shops/${shopId}/purchase`, {
      itemId,
      quantity,
    });
    return response.data;
  }

  // Inventory endpoints
  static async useItem(itemId: string, quantity: number = 1): Promise<Player> {
    const response = await api.post('/inventory/use', { itemId, quantity });
    return response.data;
  }

  static async equipItem(itemId: string): Promise<Player> {
    const response = await api.post('/inventory/equip', { itemId });
    return response.data;
  }

  static async unequipItem(slot: string): Promise<Player> {
    const response = await api.post('/inventory/unequip', { slot });
    return response.data;
  }

  // Admin endpoints
  static async adminGetAllAbilities(): Promise<Ability[]> {
    const response = await api.get('/admin/abilities');
    return response.data;
  }

  static async adminCreateClass(classData: Omit<Class, 'id'>): Promise<Class> {
    const response = await api.post('/admin/classes', classData);
    return response.data;
  }

  static async adminUpdateClass(
    classId: string,
    classData: Class
  ): Promise<Class> {
    const response = await api.patch(`/admin/classes/${classId}`, classData);
    return response.data;
  }

  static async adminDeleteClass(classId: string): Promise<void> {
    await api.delete(`/admin/classes/${classId}`);
  }

  static async adminCreateAbility(
    abilityData: Omit<Ability, 'id'>
  ): Promise<Ability> {
    const response = await api.post('/admin/abilities', abilityData);
    return response.data;
  }

  static async adminUpdateAbility(
    abilityId: string,
    abilityData: Ability
  ): Promise<Ability> {
    const response = await api.patch(
      `/admin/abilities/${abilityId}`,
      abilityData
    );
    return response.data;
  }

  static async adminDeleteAbility(abilityId: string): Promise<void> {
    await api.delete(`/admin/abilities/${abilityId}`);
  }

  static async adminAddClassAbility(
    classId: string,
    abilityId: string,
    unlockLevel: number
  ): Promise<void> {
    await api.post(`/admin/classes/${classId}/abilities`, {
      abilityId,
      unlockLevel,
    });
  }

  static async adminRemoveClassAbility(
    classId: string,
    abilityId: string
  ): Promise<void> {
    await api.delete(`/admin/classes/${classId}/abilities/${abilityId}`);
  }

  // Admin export/import
  static async adminExportConfig(): Promise<any> {
    const response = await api.get('/admin/config/export');
    return response.data;
  }

  static async adminImportConfig(configData: any): Promise<void> {
    await api.post('/admin/config/import', configData);
  }

  // ===== LOCATION MANAGEMENT =====
  static async adminGetAllLocations(): Promise<Location[]> {
    const response = await api.get('/admin/locations');
    return response.data;
  }

  static async adminCreateLocation(
    locationData: Partial<Location>
  ): Promise<Location> {
    const response = await api.post('/admin/locations', locationData);
    return response.data;
  }

  static async adminUpdateLocation(
    locationId: string,
    locationData: Partial<Location>
  ): Promise<Location> {
    const response = await api.patch(
      `/admin/locations/${locationId}`,
      locationData
    );
    return response.data;
  }

  static async adminDeleteLocation(locationId: string): Promise<void> {
    await api.delete(`/admin/locations/${locationId}`);
  }

  static async adminAddCreatureToLocation(
    locationId: string,
    creatureId: string,
    spawnRate: number
  ): Promise<void> {
    await api.post(`/admin/locations/${locationId}/creatures/${creatureId}`, {
      spawnRate,
    });
  }

  static async adminRemoveCreatureFromLocation(
    locationId: string,
    creatureId: string
  ): Promise<void> {
    await api.delete(`/admin/locations/${locationId}/creatures/${creatureId}`);
  }

  static async adminAddAdventureToLocation(
    locationId: string,
    adventureId: string,
    spawnRate: number
  ): Promise<void> {
    await api.post(`/admin/locations/${locationId}/adventures/${adventureId}`, {
      spawnRate,
    });
  }

  static async adminRemoveAdventureFromLocation(
    locationId: string,
    adventureId: string
  ): Promise<void> {
    await api.delete(
      `/admin/locations/${locationId}/adventures/${adventureId}`
    );
  }

  static async adminGetLocationCreatures(
    locationId: string
  ): Promise<{ creatures: [string, number][] }> {
    const response = await api.get(`/admin/locations/${locationId}/creatures`);
    return response.data;
  }

  static async adminGetLocationAdventures(
    locationId: string
  ): Promise<{ adventures: [string, number][] }> {
    const response = await api.get(`/admin/locations/${locationId}/adventures`);
    return response.data;
  }

  // ===== CREATURE MANAGEMENT =====
  static async adminGetAllCreatures(): Promise<any[]> {
    const response = await api.get('/admin/creatures');
    return response.data;
  }

  static async adminCreateCreature(creatureData: any): Promise<any> {
    const response = await api.post('/admin/creatures', creatureData);
    return response.data;
  }

  static async adminUpdateCreature(
    creatureId: string,
    creatureData: any
  ): Promise<any> {
    const response = await api.patch(
      `/admin/creatures/${creatureId}`,
      creatureData
    );
    return response.data;
  }

  static async adminDeleteCreature(creatureId: string): Promise<void> {
    await api.delete(`/admin/creatures/${creatureId}`);
  }

  // ===== ADVENTURE MANAGEMENT =====
  static async adminGetAllAdventures(): Promise<any[]> {
    const response = await api.get('/admin/adventures');
    return response.data;
  }

  static async adminCreateAdventure(adventureData: any): Promise<any> {
    const response = await api.post('/admin/adventures', adventureData);
    return response.data;
  }

  static async adminUpdateAdventure(
    adventureId: string,
    adventureData: any
  ): Promise<any> {
    const response = await api.patch(
      `/admin/adventures/${adventureId}`,
      adventureData
    );
    return response.data;
  }

  static async adminDeleteAdventure(adventureId: string): Promise<void> {
    await api.delete(`/admin/adventures/${adventureId}`);
  }
}

export default GameAPI;
