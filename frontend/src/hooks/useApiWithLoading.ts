import { useCallback } from 'react';
import { useLoading, LoadingKeys } from '../contexts/useLoading';
import GameAPI from '../services/api';
import type { Character } from '../types/game';

/**
 * Custom hook that provides API functions wrapped with loading states
 * This ensures consistent loading behavior across the app
 */
export const useApiWithLoading = () => {
  const { startLoading, stopLoading } = useLoading();

  // Helper function to wrap any async function with loading state
  const withLoading = useCallback(
    async <T>(key: string, asyncFn: () => Promise<T>): Promise<T> => {
      try {
        startLoading(key);
        const result = await asyncFn();
        return result;
      } finally {
        stopLoading(key);
      }
    },
    [startLoading, stopLoading]
  );

  // Authentication API calls with loading
  const auth = {
    getAuthFromCookie: () =>
      withLoading(LoadingKeys.AUTH_CHECK, () => GameAPI.getAuthFromCookie()),

    handleDiscordCallback: (code: string, state?: string) =>
      withLoading(LoadingKeys.AUTH_LOGIN, () =>
        GameAPI.handleDiscordCallback(code, state)
      ),

    logout: () => withLoading(LoadingKeys.AUTH_LOGOUT, () => GameAPI.logout()),
  };

  // User API calls with loading
  const user = {
    getUser: () => withLoading(LoadingKeys.USER_FETCH, () => GameAPI.getUser()),
  };

  // Character API calls with loading
  const characters = {
    getUserCharacters: () =>
      withLoading(LoadingKeys.CHARACTERS_FETCH, () =>
        GameAPI.getUserCharacters()
      ),

    createCharacter: (characterData: {
      userId: string;
      name: string;
      gender: string;
      classId: string;
    }) =>
      withLoading(LoadingKeys.CHARACTERS_CREATE, () =>
        GameAPI.createCharacter(characterData)
      ),

    updateCharacter: (characterId: string, updates: Partial<Character>) =>
      withLoading(LoadingKeys.CHARACTERS_UPDATE, () =>
        GameAPI.updateCharacter(characterId, updates)
      ),

    updateCharacterLastPlayed: (characterId: string) =>
      withLoading(LoadingKeys.CHARACTERS_UPDATE, () =>
        GameAPI.updateCharacterLastPlayed(characterId)
      ),
  };

  // Adventure API calls with loading
  const adventures = {
    startAdventure: (adventureId: string) =>
      withLoading(LoadingKeys.ADVENTURE_START, () =>
        GameAPI.startAdventure(adventureId)
      ),

    completeAdventure: (adventureId: string, choices?: any) =>
      withLoading(LoadingKeys.ADVENTURE_COMPLETE, () =>
        GameAPI.completeAdventure(adventureId, choices)
      ),
  };

  return {
    auth,
    user,
    characters,
    adventures,
    withLoading, // Export for custom use cases
  };
};
