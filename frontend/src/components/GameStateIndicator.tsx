import React from 'react';
import { useGameState } from '../hooks/useGameState';
import { isCombatState, isChoiceState } from '../types/gameState';
import styles from './GameStateIndicator.module.css';

export const GameStateIndicator: React.FC = () => {
  const { gameState, lastSaved, isLoading, autoSaveEnabled } = useGameState();

  const getStateDisplay = () => {
    if (isCombatState(gameState)) {
      return {
        status: 'In Combat',
        details: `${gameState.adventureName} - Turn ${gameState.turnNumber}`,
        color: '#ff0000',
      };
    } else if (isChoiceState(gameState)) {
      return {
        status: 'Making Choice',
        details: gameState.sceneName,
        color: '#ffa500',
      };
    } else {
      return {
        status: 'Exploring',
        details: 'Ready for adventure',
        color: '#00ff00',
      };
    }
  };

  const state = getStateDisplay();

  return (
    <div className={styles.indicator}>
      <div className={styles.statusRow}>
        <span className={styles.statusDot} style={{ backgroundColor: state.color }} />
        <span className={styles.statusText}>{state.status}</span>
      </div>
      <div className={styles.details}>{state.details}</div>
      {autoSaveEnabled && (
        <div className={styles.saveStatus}>
          {isLoading ? (
            <span className={styles.saving}>Saving...</span>
          ) : lastSaved ? (
            <span className={styles.saved}>
              Saved {new Date(lastSaved).toLocaleTimeString()}
            </span>
          ) : null}
        </div>
      )}
    </div>
  );
};
