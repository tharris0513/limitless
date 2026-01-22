import React, { useEffect, useState } from 'react';
import type { Ability, Character } from '../types/game';
import GameAPI from '../services/api';
import styles from './AbilitiesPage.module.css';

interface AbilitiesPageProps {
  character: Character;
  onBack?: () => void;
  onCharacterUpdate?: (character: Character) => void;
}

interface AbilityWithLevel {
  ability: Ability;
  unlockLevel: number;
}

export const AbilitiesPage: React.FC<AbilitiesPageProps> = ({ character, onBack, onCharacterUpdate }) => {
  const [abilities, setAbilities] = useState<AbilityWithLevel[]>([]);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);
  const [usingAbility, setUsingAbility] = useState<string | null>(null);

  useEffect(() => {
    loadAbilities();
  }, [character.id]);

  const loadAbilities = async () => {
    try {
      setLoading(true);
      setError(null);
      const data = await GameAPI.getCharacterUnlockedAbilities(character.id);
      setAbilities(data);
    } catch (error) {
      console.error('Failed to load abilities:', error);
      setError('Failed to load abilities. Please try again.');
    } finally {
      setLoading(false);
    }
  };

  const handleBack = () => {
    if (onBack) {
      onBack();
    }
  };

  const handleUseAbility = async (ability: Ability) => {
    if (!ability.manaCost || character.mana < ability.manaCost) {
      setError(`Not enough mana! This ability costs ${ability.manaCost} mana.`);
      setTimeout(() => setError(null), 3000);
      return;
    }

    try {
      setUsingAbility(ability.id);
      setError(null);

      const updatedCharacter = await GameAPI.activateNoncombatAbility(character.id, ability.id);

      if (onCharacterUpdate) {
        onCharacterUpdate(updatedCharacter);
      }
    } catch (error) {
      console.error('Failed to use ability:', error);
      setError('Failed to use ability. Please try again.');
    } finally {
      setUsingAbility(null);
    }
  };

  // Categorize abilities by type
  const noncombatAbilities = abilities.filter(
    a => a.ability.abilityType === 'noncombat'
  );

  const passiveAbilities = abilities.filter(
    a => a.ability.abilityType === 'passive'
  );

  const combatAbilities = abilities.filter(
    a => a.ability.abilityType === 'combat'
  );

  const renderAbilityCard = (abilityData: AbilityWithLevel, isNoncombat: boolean = false) => {
    const { ability } = abilityData;
    const activeBuff = character.activeBuffs?.find(buff => buff.abilityId === ability.id);
    const isActive = !!activeBuff;
    // Double-check: only allow use if it's truly a noncombat ability
    const isTrulyNoncombat = isNoncombat && ability.abilityType === 'noncombat';
    const canUse = isTrulyNoncombat && character.mana >= (ability.manaCost || 0);

    return (
      <div key={ability.id} className={`${styles.abilityCard} ${isActive ? styles.active : ''}`}>
        <div className={styles.abilityHeader}>
          <h3 className={styles.abilityName}>{ability.name}</h3>
          {isTrulyNoncombat && (
            <div className={styles.abilityMeta}>
              <span className={styles.manaCost}>⚡ {ability.manaCost} MP</span>
              <span className={styles.duration}>⏱ {ability.duration} adv</span>
            </div>
          )}
        </div>
        <p className={styles.abilityDescription}>{ability.description}</p>
        {isTrulyNoncombat && (
          <>
            {isActive && (
              <div className={styles.activeStatus}>
                ✓ Active ({activeBuff.remainingAdventures} adventures remaining)
              </div>
            )}
            <button
              className={`${styles.useButton} ${!canUse ? styles.disabled : ''}`}
              onClick={() => handleUseAbility(ability)}
              disabled={!canUse || usingAbility === ability.id}
            >
              {usingAbility === ability.id
                ? 'Using...'
                : isActive
                ? `Extend Duration (+${ability.duration} adv)`
                : canUse
                ? 'Use Ability'
                : 'Not Enough Mana'}
            </button>
          </>
        )}
      </div>
    );
  };

  if (loading) {
    return (
      <div className={styles.container}>
        <div className={styles.loadingMessage}>Loading abilities...</div>
      </div>
    );
  }

  if (error) {
    return (
      <div className={styles.container}>
        <div className={styles.error}>{error}</div>
        <button className={styles.backButton} onClick={handleBack}>
          ← Back to Game
        </button>
      </div>
    );
  }

  return (
    <div className={styles.container}>
      <div className={styles.header}>
        <h1 className={styles.title}>✨ Abilities</h1>
        <button className={styles.backButton} onClick={handleBack}>
          ← Back to Game
        </button>
      </div>

      {abilities.length === 0 ? (
        <div className={styles.noAbilities}>
          <p>You haven't unlocked any abilities yet.</p>
          <p>Keep leveling up to gain new abilities!</p>
        </div>
      ) : (
        <>
          {noncombatAbilities.length > 0 && (
            <div className={styles.section}>
              <h2 className={styles.sectionTitle}>Noncombat Abilities</h2>
              <div className={styles.abilitiesGrid}>
                {noncombatAbilities.map(a => renderAbilityCard(a, true))}
              </div>
            </div>
          )}

          {passiveAbilities.length > 0 && (
            <div className={styles.section}>
              <h2 className={styles.sectionTitle}>Passive Abilities</h2>
              <div className={styles.abilitiesGrid}>
                {passiveAbilities.map(a => renderAbilityCard(a, false))}
              </div>
            </div>
          )}

          {combatAbilities.length > 0 && (
            <div className={styles.section}>
              <h2 className={styles.sectionTitle}>Combat Abilities</h2>
              <div className={styles.abilitiesGrid}>
                {combatAbilities.map(a => renderAbilityCard(a, false))}
              </div>
            </div>
          )}
        </>
      )}
    </div>
  );
};
