import React, { createContext, useCallback, useEffect, useRef, useState } from 'react';
import axios from 'axios';
import type { Character } from '../types/game';
import { DEFAULT_IDLE_STATE, type GameState } from '../types/gameState';

interface GameStateContextType {
  gameState: GameState;
  setGameState: (state: GameState) => void;
  saveGameState: () => Promise<void>;
  clearGameState: () => Promise<void>;
  isLoading: boolean;
  lastSaved: Date | null;
  autoSaveEnabled: boolean;
  setAutoSaveEnabled: (enabled: boolean) => void;
}

export const GameStateContext = createContext<GameStateContextType | undefined>(undefined);

interface GameStateProviderProps {
  children: React.ReactNode;
  character: Character | null;
}

export const GameStateProvider: React.FC<GameStateProviderProps> = ({ children, character }) => {
  const [gameState, setGameStateInternal] = useState<GameState>(DEFAULT_IDLE_STATE);
  const [isLoading, setIsLoading] = useState(false);
  const [lastSaved, setLastSaved] = useState<Date | null>(null);
  const [autoSaveEnabled, setAutoSaveEnabled] = useState(true);
  const saveTimeoutRef = useRef<number | null>(null);
  const lastStateRef = useRef<string>(JSON.stringify(DEFAULT_IDLE_STATE));

  // Load game state when character changes
  useEffect(() => {
    if (character?.gameState) {
      try {
        const loadedState = JSON.parse(character.gameState) as GameState;
        setGameStateInternal(loadedState);
        lastStateRef.current = character.gameState;
      } catch (error) {
        console.error('Failed to parse game state:', error);
        setGameStateInternal(DEFAULT_IDLE_STATE);
      }
    } else {
      // Don't auto-clear if we're currently in a finished combat state
      // Let the user click the finish button to manually clear
      setGameStateInternal(prevState => {
        if ('finished' in prevState && prevState.finished) {
          return prevState; // Keep the finished combat state
        }
        return DEFAULT_IDLE_STATE;
      });
      lastStateRef.current = JSON.stringify(DEFAULT_IDLE_STATE);
    }
  }, [character?.id, character?.gameState]); // Reload when character ID or gameState changes

  // Save game state to backend
  const saveGameState = useCallback(async () => {
    if (!character?.id) {
      console.warn('Cannot save game state: no character selected');
      return;
    }

    const currentStateStr = JSON.stringify(gameState);
    
    // Don't save if state hasn't changed
    if (currentStateStr === lastStateRef.current) {
      return;
    }

    setIsLoading(true);
    try {
      const token = localStorage.getItem('authToken');
      const API_BASE_URL = import.meta.env.VITE_API_URL || 'http://localhost:8080/api';

      await axios.post(
        `${API_BASE_URL}/characters/${character.id}/state`,
        gameState,
        {
          headers: {
            Authorization: `Bearer ${token}`,
            'Content-Type': 'application/json',
          },
        }
      );

      lastStateRef.current = currentStateStr;
      setLastSaved(new Date());
      console.log('Game state saved successfully');
    } catch (error) {
      console.error('Failed to save game state:', error);
      throw error;
    } finally {
      setIsLoading(false);
    }
  }, [character?.id, gameState]);

  // Clear game state (return to idle)
  const clearGameState = useCallback(async () => {
    if (!character?.id) {
      console.warn('Cannot clear game state: no character selected');
      return;
    }

    setIsLoading(true);
    try {
      const token = localStorage.getItem('authToken');
      const API_BASE_URL = import.meta.env.VITE_API_URL || 'http://localhost:8080/api';

      await axios.delete(
        `${API_BASE_URL}/characters/${character.id}/state`,
        {
          headers: {
            Authorization: `Bearer ${token}`,
          },
        }
      );

      setGameStateInternal(DEFAULT_IDLE_STATE);
      lastStateRef.current = JSON.stringify(DEFAULT_IDLE_STATE);
      setLastSaved(new Date());
      console.log('Game state cleared successfully');
    } catch (error) {
      console.error('Failed to clear game state:', error);
      throw error;
    } finally {
      setIsLoading(false);
    }
  }, [character?.id]);

  // Auto-save with debounce
  useEffect(() => {
    if (!autoSaveEnabled || !character?.id) {
      return;
    }

    // Clear existing timeout
    if (saveTimeoutRef.current) {
      clearTimeout(saveTimeoutRef.current);
    }

    // Set new timeout to save after 2 seconds of no changes
    saveTimeoutRef.current = setTimeout(() => {
      saveGameState().catch(console.error);
    }, 2000);

    return () => {
      if (saveTimeoutRef.current) {
        clearTimeout(saveTimeoutRef.current);
      }
    };
  }, [gameState, autoSaveEnabled, character?.id, saveGameState]);

  // Save on page unload
  useEffect(() => {
    const handleBeforeUnload = () => {
      if (autoSaveEnabled && character?.id) {
        // Use synchronous beacon API for reliable save on page close
        const currentStateStr = JSON.stringify(gameState);
        if (currentStateStr !== lastStateRef.current) {
          const token = localStorage.getItem('authToken');
          const API_BASE_URL = import.meta.env.VITE_API_URL || 'http://localhost:8080/api';
          
          // Note: sendBeacon only works with small payloads
          // For larger states, consider using keepalive fetch
          const blob = new Blob([currentStateStr], { type: 'application/json' });
          navigator.sendBeacon(
            `${API_BASE_URL}/characters/${character.id}/state?token=${token}`,
            blob
          );
        }
      }
    };

    window.addEventListener('beforeunload', handleBeforeUnload);
    return () => window.removeEventListener('beforeunload', handleBeforeUnload);
  }, [gameState, autoSaveEnabled, character?.id]);

  const setGameState = useCallback((state: GameState) => {
    setGameStateInternal(state);
  }, []);

  return (
    <GameStateContext.Provider
      value={{
        gameState,
        setGameState,
        saveGameState,
        clearGameState,
        isLoading,
        lastSaved,
        autoSaveEnabled,
        setAutoSaveEnabled,
      }}
    >
      {children}
    </GameStateContext.Provider>
  );
};
