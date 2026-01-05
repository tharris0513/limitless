import React, { useState } from 'react';
import { useGameState } from '../hooks/useGameState';
import { isCombatState } from '../types/gameState';
import type { Character } from '../types/game';
import GameAPI from '../services/api';
import styles from './CombatPage.module.css';

interface CombatPageProps {
  character: Character;
  onCharacterUpdate?: (character: Character) => void;
}

export const CombatPage: React.FC<CombatPageProps> = ({ character, onCharacterUpdate }) => {
  const { gameState, clearGameState } = useGameState();
  const [fleeing, setFleeing] = useState(false);

  if (!isCombatState(gameState)) {
    return null;
  }

  const { enemy, turnNumber, playerHealth, playerMana, combatLog } = gameState;

  const handleFlee = async () => {
    if (fleeing) return;
    
    setFleeing(true);
    try {
      const updatedCharacter = await GameAPI.fleeCombat(character.id);
      await clearGameState();
      
      // Notify parent component of character update
      if (onCharacterUpdate) {
        onCharacterUpdate(updatedCharacter);
      }
    } catch (error) {
      console.error('Failed to flee:', error);
      alert('Failed to flee from combat. Please try again.');
      setFleeing(false);
    }
  };

  return (
    <div className={styles.combatContainer}>
      <div className={styles.header}>
        <h1 className={styles.title}>⚔️ Combat</h1>
        <div className={styles.turnInfo}>Turn {turnNumber}</div>
      </div>

      {/* Introduction Text - shown on turn 1 */}
      {turnNumber === 1 && enemy.introductionText && (
        <div className={styles.introductionBox}>
          <p className={styles.introductionText}>{enemy.introductionText}</p>
        </div>
      )}

      <div className={styles.combatArea}>
        {/* Player Status */}
        <div className={styles.playerSection}>
          <h2 className={styles.sectionTitle}>{character.name}</h2>
          <div className={styles.statsGrid}>
            <div className={styles.stat}>
              <span className={styles.statLabel}>❤️ Health:</span>
              <span className={styles.statValue}>
                {playerHealth} / {character.health}
              </span>
            </div>
            <div className={styles.stat}>
              <span className={styles.statLabel}>⭐ Mana:</span>
              <span className={styles.statValue}>
                {playerMana} / {character.mana}
              </span>
            </div>
          </div>
        </div>

        {/* Enemy Status */}
        <div className={styles.enemySection}>
          <h2 className={styles.sectionTitle}>
            {enemy.name} (Lvl {enemy.level})
          </h2>
          <div className={styles.statsGrid}>
            <div className={styles.stat}>
              <span className={styles.statLabel}>❤️ Health:</span>
              <span className={styles.statValue}>
                {enemy.health} / {enemy.maxHealth}
              </span>
            </div>
            <div className={styles.healthBar}>
              <div
                className={styles.healthFill}
                style={{
                  width: `${(enemy.health / enemy.maxHealth) * 100}%`,
                }}
              />
            </div>
          </div>
        </div>
      </div>

      {/* Combat Actions */}
      <div className={styles.actionsSection}>
        <h3 className={styles.actionsTitle}>Actions</h3>
        <div className={styles.actionButtons}>
          <button className={styles.actionButton}>
            🗡️ Attack
          </button>
          <button className={styles.actionButton}>
            🛡️ Defend
          </button>
          <button className={styles.actionButton}>
            ✨ Use Ability
          </button>
          <button 
            className={styles.actionButton}
            onClick={handleFlee}
            disabled={fleeing}
          >
            🏃 {fleeing ? 'Fleeing...' : 'Flee'}
          </button>
        </div>
      </div>

      {/* Combat Log */}
      {combatLog && combatLog.length > 0 && (
        <div className={styles.combatLog}>
          <h3 className={styles.logTitle}>Combat Log</h3>
          <div className={styles.logEntries}>
            {combatLog.map((entry, index) => (
              <div key={index} className={styles.logEntry}>
                <span className={styles.logTurn}>Turn {entry.turn}:</span>
                <span className={styles.logMessage}>{entry.message}</span>
              </div>
            ))}
          </div>
        </div>
      )}
    </div>
  );
};
