import { useCallback } from 'react';
import { useGameState } from './useGameState';
import axios from 'axios';
import type { CombatState, ChoiceState, EnemyState } from '../types/gameState';

/**
 * Hook to manage combat state
 */
export const useCombatState = () => {
  const { gameState, setGameState, saveGameState } = useGameState();

  const startCombat = useCallback(
    async (
      adventureId: string,
      adventureName: string,
      enemy: EnemyState,
      playerHealth: number,
      playerMana: number,
      characterId: string
    ) => {
      const combatState: CombatState = {
        inCombat: true,
        status: 'started',
        adventureId,
        adventureName,
        turnNumber: 1,
        playerHealth,
        playerMana,
        enemy,
        combatLog: [],
      };
      setGameState(combatState);

      // Immediately save the combat state to backend (don't wait for debounce)
      try {
        const token = localStorage.getItem('authToken');
        const API_BASE_URL =
          import.meta.env.VITE_API_URL || 'http://localhost:8080/api';

        await axios.post(
          `${API_BASE_URL}/characters/${characterId}/state`,
          combatState,
          {
            headers: {
              Authorization: `Bearer ${token}`,
              'Content-Type': 'application/json',
            },
          }
        );
        console.log('Combat state initialized and saved to backend');
      } catch (error) {
        console.error('Failed to save initial combat state:', error);
        throw error;
      }
    },
    [setGameState]
  );

  const updateCombatState = useCallback(
    (updates: Partial<CombatState>) => {
      if ('inCombat' in gameState && gameState.inCombat) {
        setGameState({ ...gameState, ...updates });
      }
    },
    [gameState, setGameState]
  );

  const endCombat = useCallback(() => {
    setGameState({ inCombat: false, inChoice: false });
    saveGameState();
  }, [setGameState, saveGameState]);

  return {
    isCombat: 'inCombat' in gameState && gameState.inCombat === true,
    combatState:
      'inCombat' in gameState && gameState.inCombat === true ? gameState : null,
    startCombat,
    updateCombatState,
    endCombat,
  };
};

/**
 * Hook to manage choice/dialogue state
 */
export const useChoiceState = () => {
  const { gameState, setGameState, saveGameState } = useGameState();

  const startChoice = useCallback(
    (
      adventureId: string,
      adventureName: string,
      sceneId: string,
      sceneName: string,
      sceneDescription: string,
      choices: ChoiceState['choices']
    ) => {
      const choiceState: ChoiceState = {
        inChoice: true,
        adventureId,
        adventureName,
        sceneId,
        sceneName,
        sceneDescription,
        choices,
      };
      setGameState(choiceState);
    },
    [setGameState]
  );

  const recordChoice = useCallback(
    (choiceId: string) => {
      if ('inChoice' in gameState && gameState.inChoice) {
        const previousChoices = gameState.previousChoices || [];
        setGameState({
          ...gameState,
          previousChoices: [
            ...previousChoices,
            {
              sceneId: gameState.sceneId,
              choiceId,
              timestamp: new Date().toISOString(),
            },
          ],
        });
      }
    },
    [gameState, setGameState]
  );

  const endChoice = useCallback(() => {
    setGameState({ inCombat: false, inChoice: false });
    saveGameState();
  }, [setGameState, saveGameState]);

  return {
    isChoice: 'inChoice' in gameState && gameState.inChoice === true,
    choiceState:
      'inChoice' in gameState && gameState.inChoice === true ? gameState : null,
    startChoice,
    recordChoice,
    endChoice,
  };
};
