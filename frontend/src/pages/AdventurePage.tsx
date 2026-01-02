import React, { useEffect } from 'react';
import { useNavigate } from 'react-router-dom';
import { useGameState } from '../hooks/useGameState';
import { isChoiceState } from '../types/gameState';
import type { Character } from '../types/game';
import styles from './AdventurePage.module.css';

interface AdventurePageProps {
  character: Character;
}

export const AdventurePage: React.FC<AdventurePageProps> = ({ character }) => {
  const { gameState } = useGameState();
  const navigate = useNavigate();

  // Redirect if not in adventure
  useEffect(() => {
    if (!isChoiceState(gameState)) {
      navigate('/');
    }
  }, [gameState, navigate]);

  if (!isChoiceState(gameState)) {
    return null;
  }

  const { adventureName, sceneName, sceneDescription, choices, previousChoices } = gameState;

  const handleChoice = (choiceId: string) => {
    console.log('Selected choice:', choiceId);
    // TODO: Implement choice handling logic
  };

  return (
    <div className={styles.adventureContainer}>
      <div className={styles.header}>
        <h1 className={styles.title}>📜 {adventureName}</h1>
      </div>

      <div className={styles.sceneSection}>
        <h2 className={styles.sceneName}>{sceneName}</h2>
        <div className={styles.sceneDescription}>
          {sceneDescription}
        </div>
      </div>

      <div className={styles.choicesSection}>
        <h3 className={styles.choicesTitle}>What do you do?</h3>
        <div className={styles.choicesList}>
          {choices.map((choice) => (
            <button
              key={choice.id}
              className={styles.choiceButton}
              onClick={() => handleChoice(choice.id)}
            >
              <div className={styles.choiceText}>{choice.text}</div>
              {choice.description && (
                <div className={styles.choiceDescription}>
                  {choice.description}
                </div>
              )}
              {choice.requirements && (
                <div className={styles.choiceRequirements}>
                  {choice.requirements.stat && (
                    <span>Requires {choice.requirements.stat} ≥ {choice.requirements.minValue}</span>
                  )}
                  {choice.requirements.item && (
                    <span>Requires item: {choice.requirements.item}</span>
                  )}
                </div>
              )}
            </button>
          ))}
        </div>
      </div>

      {previousChoices && previousChoices.length > 0 && (
        <div className={styles.historySection}>
          <h3 className={styles.historyTitle}>Previous Choices</h3>
          <div className={styles.historyList}>
            {previousChoices.map((prev, index) => (
              <div key={index} className={styles.historyEntry}>
                Scene: {prev.sceneId} - Choice: {prev.choiceId}
              </div>
            ))}
          </div>
        </div>
      )}

      <div className={styles.characterInfo}>
        <h3 className={styles.infoTitle}>Character Info</h3>
        <div className={styles.infoGrid}>
          <div className={styles.infoStat}>
            <span className={styles.infoLabel}>Name:</span>
            <span className={styles.infoValue}>{character.name}</span>
          </div>
          <div className={styles.infoStat}>
            <span className={styles.infoLabel}>Level:</span>
            <span className={styles.infoValue}>{character.level}</span>
          </div>
        </div>
      </div>
    </div>
  );
};
