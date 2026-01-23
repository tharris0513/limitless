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
  const [_defeat, setDefeat] = useState(false);
  const [_victoryMessage, setVictoryMessage] = useState<string>('');
  const [abilities, setAbilities] = useState<Ability[]>([]);
  const combatLogRef = useRef<HTMLDivElement>(null);

  // Initialize from character.gameState if context is empty but character has combat state
  useEffect(() => {
    if (!isCombatState(gameState) && character.gameState) {
      try {
        const parsedState = JSON.parse(character.gameState);
        if (parsedState && parsedState.inCombat) {
          console.log('Initializing combat state from character.gameState');
          setGameState(parsedState);
        }
      } catch (error) {
        console.error('Failed to parse character game state:', error);
      }
    }
  }, [character.gameState, gameState, setGameState]);

  // Fetch character abilities on mount
  useEffect(() => {
    const fetchAbilities = async () => {
      try {
        // Get character's unlocked abilities with full ability data
        const unlockedAbilities = await GameAPI.getCharacterUnlockedAbilities(character.id);
        console.log('Unlocked abilities response:', unlockedAbilities);
        
        // Filter to only combat abilities
        const activeAbilities = unlockedAbilities
          .filter(ua => ua.ability.abilityType === 'combat')
          .map(ua => ua.ability)
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
      const result = await GameAPI.performCombatAction(character.id, { actionType: 'melee' });
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
      
      // Check for defeat
      if (result.defeat) {
        setDefeat(true);
        if (result.defeatMessage) {
          setCombatText(prev => [...prev, { text: result.defeatMessage!, type: 'enemy' }]);
        }
        
        // Update game state to mark combat as finished
        if (result.gameState) {
          setGameState(result.gameState);
        }
        
        // Refresh character to update HP and adventures in sidebar
        if (onCharacterUpdate) {
          try {
            const updatedChar = await GameAPI.getCharacter(character.id);
            onCharacterUpdate(updatedChar);
          } catch (error) {
            console.error('Failed to refresh character after defeat:', error);
          }
        }
        
        setAttacking(false);
        return;
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
          
          // Add ability learned messages
          if (result.levelUp.abilitiesLearned && result.levelUp.abilitiesLearned.length > 0) {
            result.levelUp.abilitiesLearned.forEach(ability => {
              const abilityMsg = `✨ You learned a new ability: ${ability.name}! ${ability.description}`;
              setCombatText(prev => [...prev, { text: abilityMsg, type: 'player' }]);
            });
          }
        }
        
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

  const handleUseAbility = async (abilityId: string) => {
    if (attacking || !isCombatState(gameState)) return;
    
    setAttacking(true);
    try {
      const result = await GameAPI.performCombatAction(character.id, { 
        actionType: 'ability', 
        abilityId 
      });
      console.log('Ability use result:', result);
      
      // Add ability attack descriptions to combat text
      const playerAttacks = result.attacks.map(attack => ({
        text: attack.description,
        type: 'player' as const
      }));
      setCombatText(prev => [...prev, ...playerAttacks]);
      
      // Add enemy counterattacks to combat text
      if (result.enemyAttacks && result.enemyAttacks.length > 0) {
        const enemyAttacks = result.enemyAttacks.map(attack => ({
          text: attack.description,
          type: 'enemy' as const
        }));
        setCombatText(prev => [...prev, ...enemyAttacks]);
      }
      
      // Check for defeat
      if (result.defeat) {
        setDefeat(true);
        if (result.defeatMessage) {
          setCombatText(prev => [...prev, { text: result.defeatMessage!, type: 'enemy' }]);
        }
        
        // Update game state to mark combat as finished
        if (result.gameState) {
          setGameState(result.gameState);
        }
        
        // Refresh character to update HP, mana, and adventures in sidebar
        if (onCharacterUpdate) {
          try {
            const updatedChar = await GameAPI.getCharacter(character.id);
            onCharacterUpdate(updatedChar);
          } catch (error) {
            console.error('Failed to refresh character after defeat:', error);
          }
        }
        
        setAttacking(false);
        return;
      }
      
      // Update game state (includes enemy health and updated mana)
      if (result.gameState) {
        setGameState(result.gameState);
      }
      
      // Refresh character to update HP and mana in sidebar
      if (onCharacterUpdate) {
        try {
          const updatedChar = await GameAPI.getCharacter(character.id);
          onCharacterUpdate(updatedChar);
        } catch (error) {
          console.error('Failed to refresh character after ability use:', error);
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
          
          // Add ability learned messages
          if (result.levelUp.abilitiesLearned && result.levelUp.abilitiesLearned.length > 0) {
            result.levelUp.abilitiesLearned.forEach(ability => {
              const abilityMsg = `✨ You learned a new ability: ${ability.name}! ${ability.description}`;
              setCombatText(prev => [...prev, { text: abilityMsg, type: 'player' }]);
            });
          }
        }
        
        // Refresh character
        if (onCharacterUpdate) {
          try {
            const updatedChar = await GameAPI.getCharacter(character.id);
            onCharacterUpdate(updatedChar);
          } catch (error) {
            console.error('Failed to refresh character after victory:', error);
          }
        }
      }
      
    } catch (error: any) {
      console.error('Failed to use ability:', error);
      // Show user-friendly error message
      const errorMsg = error?.response?.data?.error || 'Failed to use ability. Please try again.';
      alert(errorMsg);
    } finally {
      setAttacking(false);
    }
  };

  const handleFinish = async () => {
    const updatedCharacter = await clearGameState();
    if (updatedCharacter && onCharacterUpdate) {
      onCharacterUpdate(updatedCharacter);
    }
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
          {(gameState.status === 'victory' || gameState.status === 'defeat') ? (
            <button
              className={styles.actionButton}
              onClick={handleFinish}
              style={{ gridColumn: '1 / -1' }}
            >
              ✓ {gameState.status === 'victory' ? 'Finish' : 'Return'}
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
      {gameState.status !== 'victory' && gameState.status !== 'defeat' && abilities.length > 0 && (
        <div className={styles.hotbarSection}>
          <h3 className={styles.hotbarTitle}>Abilities</h3>
          <div className={styles.hotbar}>
            {abilities.map((ability) => {
              const cooldownTurns = gameState.abilityCooldowns?.[ability.id] || 0;
              const onCooldown = cooldownTurns > 0;
              const notEnoughMana = character.mana < ability.manaCost;
              
              return (
                <button
                  key={ability.id}
                  className={styles.hotbarButton}
                  disabled={onCooldown || notEnoughMana || attacking}
                  onClick={() => handleUseAbility(ability.id)}
                  title={`${ability.description}\nMana Cost: ${ability.manaCost}${ability.cooldown > 0 ? `\nCooldown: ${ability.cooldown} turns` : ''}${onCooldown ? `\nOn cooldown for ${cooldownTurns} more turn${cooldownTurns === 1 ? '' : 's'}` : ''}`}
                >
                  <div className={styles.abilityName}>{ability.name}</div>
                  <div className={styles.abilityMana}>
                    {onCooldown ? (
                      <span className={styles.cooldownBadge}>{cooldownTurns}</span>
                    ) : (
                      `${ability.manaCost} MP`
                    )}
                  </div>
                </button>
              );
            })}
          </div>
        </div>
      )}
    </div>
  );
};
