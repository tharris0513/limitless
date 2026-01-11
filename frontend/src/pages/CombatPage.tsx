import React, { useState, useRef, useEffect } from 'react';
import { useGameState } from '../hooks/useGameState';
import { isCombatState } from '../types/gameState';
import type { Character, Ability } from '../types/game';
import GameAPI from '../services/api';
import { parseFormattedText } from '../utils/formatText';
import styles from './CombatPage.module.css';

interface CombatPageProps {
  character: Character;
  onCharacterUpdate?: (character: Character) => void;
}

export const CombatPage: React.FC<CombatPageProps> = ({ character, onCharacterUpdate }) => {
  const { gameState, setGameState, clearGameState } = useGameState();
  const [fleeing, setFleeing] = useState(false);
  const [attacking, setAttacking] = useState(false);
  const [combatText, setCombatText] = useState<Array<{ text: string; type: 'player' | 'enemy' }>>([]);
  const [victory, setVictory] = useState(false);
  const [_victoryMessage, setVictoryMessage] = useState<string>('');
  const [abilities, setAbilities] = useState<Ability[]>([]);
  const combatLogRef = useRef<HTMLDivElement>(null);

  // Fetch character abilities on mount
  useEffect(() => {
    const fetchAbilities = async () => {
      try {
        // Get character's unlocked ability IDs
        const characterAbilities = await GameAPI.getCharacterAbilities(character.id);
        console.log('Character abilities response:', characterAbilities);
        console.log('Character abilities type:', typeof characterAbilities, Array.isArray(characterAbilities));
        
        const unlockedAbilityIds = new Set(characterAbilities.map((ca: any) => ca.ability_id || ca.abilityId));
        console.log('Unlocked ability IDs:', Array.from(unlockedAbilityIds));
        
        // Get all abilities for the character's class
        console.log('Fetching class abilities for class:', character.classId);
        const classAbilities = await GameAPI.getClassAbilities(character.classId);
        console.log('Class abilities response:', classAbilities);
        
        // Filter to only unlocked active abilities
        const activeAbilities = classAbilities
          .filter(ca => {
            const hasAbility = unlockedAbilityIds.has(ca.ability.id);
            console.log(`Checking ability ${ca.ability.id} (${ca.ability.name}): unlocked=${hasAbility}, type=${ca.ability.abilityType}`);
            return hasAbility;
          })
          .filter(ca => ca.ability.abilityType === 'active')
          .map(ca => ca.ability)
          .reverse(); // Show lowest level abilities first
        
        console.log('Active abilities:', activeAbilities);
        setAbilities(activeAbilities);
      } catch (error) {
        console.error('Failed to fetch abilities:', error);
      }
    };
    fetchAbilities();
  }, [character.id, character.classId]);

  // Auto-scroll combat log to bottom when new entries are added
  useEffect(() => {
    if (combatLogRef.current) {
      combatLogRef.current.scrollTop = combatLogRef.current.scrollHeight;
    }
  }, [combatText]);

  if (!isCombatState(gameState)) {
    return null;
  }

  const { turnNumber, combatLog } = gameState;

  // Debug logging
  console.log('CombatPage gameState:', gameState);
  console.log('Enemy health:', gameState.enemy.health, 'Max:', gameState.enemy.maxHealth);

  const handleAttack = async () => {
    if (attacking || !isCombatState(gameState)) return;
    
    setAttacking(true);
    try {
      const result = await GameAPI.performAttack(character.id);
      console.log('Attack result:', result);
      console.log('Level up data:', result.levelUp);
      
      // Add attack descriptions to combat text (player attacks in green)
      const playerAttacks = result.attacks.map(attack => ({
        text: attack.description,
        type: 'player' as const
      }));
      setCombatText(prev => [...prev, ...playerAttacks]);
      
      // Add enemy counterattacks to combat text (enemy attacks in red)
      if (result.enemyAttacks && result.enemyAttacks.length > 0) {
        const enemyAttacks = result.enemyAttacks.map(attack => ({
          text: attack.description,
          type: 'enemy' as const
        }));
        setCombatText(prev => [...prev, ...enemyAttacks]);
      }
      
      // Update game state (this includes enemy health at 0 on victory)
      if (result.gameState) {
        setGameState(result.gameState);
      }
      
      // Refresh character to update HP in sidebar
      if (onCharacterUpdate) {
        try {
          const updatedChar = await GameAPI.getCharacter(character.id);
          onCharacterUpdate(updatedChar);
        } catch (error) {
          console.error('Failed to refresh character after attack:', error);
          // Don't show error to user, combat continues
        }
      }
      
      // Check for victory
      if (result.victory) {
        setVictory(true);
        if (result.victoryMessage) {
          setVictoryMessage(result.victoryMessage);
          setCombatText(prev => [...prev, { text: result.victoryMessage!, type: 'player' }]);
        }
        
        // Add level-up message if character leveled up
        if (result.levelUp) {
          const levelUpMsg = `🎉 Level Up! You are now level ${result.levelUp.newLevel}! ` +
            `+${result.levelUp.statIncreases.might} Might, ` +
            `+${result.levelUp.statIncreases.defense} Defense, ` +
            `+${result.levelUp.statIncreases.magic} Magic, ` +
            `+${result.levelUp.statIncreases.resistance} Resistance, ` +
            `+${result.levelUp.statIncreases.agility} Agility, ` +
            `+${result.levelUp.statIncreases.maxHealth} Max Health, ` +
            `+${result.levelUp.statIncreases.maxMana} Max Mana`;
          setCombatText(prev => [...prev, { text: levelUpMsg, type: 'player' }]);
        }
        
        // Don't clear game state yet - let user click Finish button
        // The backend has already cleared it, but we keep it locally to show victory screen
        
        // Character has already been updated on backend, just need to refresh
        if (onCharacterUpdate) {
          try {
            const updatedChar = await GameAPI.getCharacter(character.id);
            console.log('Updated character from API:', updatedChar);
            onCharacterUpdate(updatedChar);
          } catch (error) {
            console.error('Failed to refresh character after victory:', error);
            // Don't show error to user, victory already succeeded
          }
        }
      }
      
    } catch (error) {
      console.error('Failed to attack:', error);
      // Only show error if not in victory state
      if (!victory) {
        alert('Failed to attack. Please try again.');
      }
    } finally {
      setAttacking(false);
    }
  };

  const handleFinish = async () => {
    await clearGameState();
  };

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

      {/* Enemy Status */}
      <div className={styles.enemySection}>
        <h2 className={styles.sectionTitle}>
          {gameState.enemy.name} (Lvl {gameState.enemy.level})
        </h2>
        <div className={styles.statsGrid}>
          <div className={styles.stat}>
            <span className={styles.statLabel}>❤️ Health:</span>
            <span className={styles.statValue}>
              {gameState.enemy.health ?? 0} / {gameState.enemy.maxHealth ?? 0}
            </span>
          </div>
          <div className={styles.healthBar}>
            <div
              className={styles.healthFill}
              style={{
                width: `${((gameState.enemy.health ?? 0) / (gameState.enemy.maxHealth || 1)) * 100}%`,
              }}
            />
          </div>
        </div>
      </div>

      {/* Combat Text Log */}
      <div className={styles.combatLog} ref={combatLogRef}>
        {/* Introduction Text */}
        {gameState.enemy.introductionText && (
          <div className={styles.logEntry}>
            <p className={styles.introductionText}>{gameState.enemy.introductionText}</p>
          </div>
        )}
        
        {/* Combat Log Entries */}
        {combatLog && combatLog.length > 0 && (
          <div className={styles.logEntries}>
            {combatLog.map((entry, index) => (
              <div key={index} className={styles.logEntry}>
                <span className={styles.logTurn}>Turn {entry.turn}:</span>
                <span className={styles.logMessage}>{entry.message}</span>
              </div>
            ))}
          </div>
        )}
        
        {/* Attack Text */}
        {combatText.length > 0 && (
          <div className={styles.logEntries}>
            {combatText.map((entry, index) => (
              <div 
                key={`attack-${index}`} 
                className={entry.type === 'enemy' ? styles.enemyLogEntry : styles.logEntry}
              >
                <span className={styles.logMessage}>{parseFormattedText(entry.text)}</span>
              </div>
            ))}
          </div>
        )}
      </div>

      {/* Combat Actions */}
      <div className={styles.actionsSection}>
        <h3 className={styles.actionsTitle}>Actions</h3>
        <div className={styles.actionButtons}>
          {victory ? (
            <button
              className={styles.actionButton}
              onClick={handleFinish}
              style={{ gridColumn: '1 / -1' }}
            >
              ✓ Finish
            </button>
          ) : (
            <>
              <button 
                className={styles.actionButton}
                onClick={handleAttack}
                disabled={attacking}
              >
                🗡️ {attacking ? 'Attacking...' : 'Attack'}
              </button>
              <button className={styles.actionButton}>
                🛡️ Defend
              </button>
              <button 
                className={styles.actionButton}
                onClick={handleFlee}
                disabled={fleeing}
              >
                🏃 {fleeing ? 'Fleeing...' : 'Flee'}
              </button>
            </>
          )}
        </div>
      </div>

      {/* Ability Hotbar */}
      {!victory && abilities.length > 0 && (
        <div className={styles.hotbarSection}>
          <h3 className={styles.hotbarTitle}>Abilities</h3>
          <div className={styles.hotbar}>
            {abilities.map((ability) => (
              <button
                key={ability.id}
                className={styles.hotbarButton}
                disabled={character.mana < ability.manaCost}
                title={`${ability.description}\nMana Cost: ${ability.manaCost}${ability.cooldown > 0 ? `\nCooldown: ${ability.cooldown} turns` : ''}`}
              >
                <div className={styles.abilityName}>{ability.name}</div>
                <div className={styles.abilityMana}>{ability.manaCost} MP</div>
              </button>
            ))}
          </div>
        </div>
      )}
    </div>
  );
};
