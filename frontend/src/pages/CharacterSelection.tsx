import React from 'react';
import { type Character, type User } from '../types/game';
import { Plus, Swords, Star } from 'lucide-react';
import { CardSkeleton } from '../components/LoadingStates';
import styles from './CharacterSelection.module.css';

interface CharacterSelectionProps {
  user: User;
  characters: Character[];
  isLoading?: boolean;
  onCharacterSelect: (character: Character) => void;
  onCreateNewCharacter: () => void;
}

export const CharacterSelection: React.FC<CharacterSelectionProps> = ({
  user,
  characters,
  isLoading = false,
  onCharacterSelect,
  onCreateNewCharacter,
}) => {
  return (
    <div className={styles.container}>
      <div className={styles.header}>
        <h2 className={styles.title}>
          ⚔️ SELECT CHARACTER
        </h2>
        <p className={styles.welcome}>
          Welcome back, <span className={styles.username}>{user.username}</span>!
        </p>
        <p className={styles.subtitle}>
          Choose a character to continue your adventure in the Limitless realm.
        </p>
      </div>

      <div className={styles.charactersGrid}>
        {isLoading ? (
          // Show skeleton loaders while loading
          Array.from({ length: 3 }, (_, index) => (
            <CardSkeleton key={index} height="120px" />
          ))
        ) : (
          characters.map((character) => (
            <div
              key={character.id}
              className={styles.characterCard}
              onClick={() => onCharacterSelect(character)}
            >
              <div className={styles.characterInfo}>
                <h3 className={styles.characterName}>{character.name}</h3>
                <div className={styles.characterStats}>
                  <div className={styles.level}>
                    <Star size={16} /> Level {character.level}
                  </div>
                  <div className={styles.location}>
                    📍 {character.location}
                  </div>
                </div>
                <div className={styles.lastPlayed}>
                  Last played: {new Date(character.lastPlayed).toLocaleDateString()}
                </div>
              </div>
              <div className={styles.characterActions}>
                <Swords size={20} />
                <span>CONTINUE</span>
              </div>
            </div>
          ))
        )}

        {!isLoading && (
          <div
            className={styles.newCharacterCard}
            onClick={onCreateNewCharacter}
          >
            <div className={styles.newCharacterIcon}>
              <Plus size={40} />
            </div>
            <div className={styles.newCharacterText}>
              <h3>CREATE NEW CHARACTER</h3>
              <p>Start a fresh adventure</p>
            </div>
          </div>
        )}
      </div>

      {characters.length === 0 && !isLoading && (
        <div className={styles.noCharacters}>
          <p>You don't have any characters yet.</p>
          <p>Create your first character to begin your adventure!</p>
        </div>
      )}
    </div>
  );
};