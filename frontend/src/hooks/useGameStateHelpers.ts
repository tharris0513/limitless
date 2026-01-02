import { useCallback } from 'react';
import { useGameState } from './useGameState';
import type { CombatState, ChoiceState, EnemyState } from '../types/gameState';

/**
 * Hook to manage combat state
 */
export const useCombatState = () => {
  const { gameState, setGameState, saveGameState } = useGameState();

  const startCombat = useCallback(
    (
      adventureId: string,
      adventureName: string,
      enemy: EnemyState,
      playerHealth: number,
      playerMana: number
    ) => {
      const combatState: CombatState = {
        inCombat: true,
        adventureId,
        adventureName,
        turnNumber: 1,
        playerHealth,
        playerMana,
        enemy,
        combatLog: [],
      };
      setGameState(combatState);
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
