import React, { useEffect, useState } from 'react';
import type { Ability, Character } from '../types/game';
import GameAPI from '../services/api';
import styles from './AbilitiesPage.module.css';

interface AbilitiesPageProps {
  character: Character;
  onBack?: () => void;
}

interface AbilityWithLevel {
  ability: Ability;
  unlockLevel: number;
}

export const AbilitiesPage: React.FC<AbilitiesPageProps> = ({ character, onBack }) => {
  const [abilities, setAbilities] = useState<AbilityWithLevel[]>([]);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);

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

  // Categorize abilities by type
  const noncombatAbilities = abilities.filter(
    a => a.ability.abilityType !== 'combat' && a.ability.abilityType !== 'passive'
  );

  const passiveAbilities = abilities.filter(
    a => a.ability.abilityType === 'passive'
  );

  const combatAbilities = abilities.filter(
    a => a.ability.abilityType === 'combat'
  );

  const renderAbilityCard = (abilityData: AbilityWithLevel) => {
    const { ability } = abilityData;

    return (
      <div key={ability.id} className={styles.abilityCard}>
        <h3 className={styles.abilityName}>{ability.name}</h3>
        <p className={styles.abilityDescription}>{ability.description}</p>
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
                {noncombatAbilities.map(renderAbilityCard)}
              </div>
            </div>
          )}

          {passiveAbilities.length > 0 && (
            <div className={styles.section}>
              <h2 className={styles.sectionTitle}>Passive Abilities</h2>
              <div className={styles.abilitiesGrid}>
                {passiveAbilities.map(renderAbilityCard)}
              </div>
            </div>
          )}

          {combatAbilities.length > 0 && (
            <div className={styles.section}>
              <h2 className={styles.sectionTitle}>Combat Abilities</h2>
              <div className={styles.abilitiesGrid}>
                {combatAbilities.map(renderAbilityCard)}
              </div>
            </div>
          )}
        </>
      )}
    </div>
  );
};
