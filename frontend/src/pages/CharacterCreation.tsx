import React, { useState } from 'react';
import { LoadingButton } from '../components/LoadingStates';
import { useApiWithLoading } from '../hooks/useApiWithLoading';
import type { Character, User } from '../types/game';
import styles from './CharacterCreation.module.css';

interface CharacterCreationProps {
  user: User;
  onCharacterCreated: (character: Character) => void;
}

export const CharacterCreation: React.FC<CharacterCreationProps> = ({
  user,
  onCharacterCreated,
}) => {
  const api = useApiWithLoading();
  const [formData, setFormData] = useState({
    name: '',
    gender: 'Male',
  });
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState('');

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault();

    if (!formData.name.trim()) {
      setError('Please enter a character name');
      return;
    }

    if (formData.name.length < 3) {
      setError('Character name must be at least 3 characters long');
      return;
    }

    try {
      setLoading(true);
      setError('');

      // Create character via API
      const newCharacter = await api.characters.createCharacter({
        userId: user.id,
        name: formData.name,
        gender: formData.gender,
      });

      console.log('Created character:', newCharacter);
      onCharacterCreated(newCharacter);
    } catch (err: unknown) {
      console.error('Character creation error:', err);
      setError('Failed to create character. Please try again.');
    } finally {
      setLoading(false);
    }
  };

  const handleChange = (
    e: React.ChangeEvent<HTMLInputElement | HTMLSelectElement>
  ) => {
    setFormData(prev => ({
      ...prev,
      [e.target.name]: e.target.value,
    }));
  };

  return (
    <div className={styles.characterContainer}>
      <h1 className={styles.title}>⚔️ Create Character</h1>

      <div className={styles.welcomeText}>
        Welcome, <span className={styles.highlight}>adventurer</span>!<br />
        Before you begin your journey in the Limitless realm,
        <br />
        you must create your character.
      </div>

      <form onSubmit={handleSubmit} className={styles.form}>
        <div className={styles.formGroup}>
          <label htmlFor="name" className={styles.label}>
            Character Name
          </label>
          <input
            type="text"
            id="name"
            name="name"
            value={formData.name}
            onChange={handleChange}
            placeholder="Enter your character's name"
            className={styles.input}
            maxLength={20}
            required
            disabled={loading}
          />
        </div>

        <div className={styles.formGroup}>
          <label htmlFor="gender" className={styles.label}>
            Gender
          </label>
          <select
            id="gender"
            name="gender"
            value={formData.gender}
            onChange={handleChange}
            className={styles.select}
            disabled={loading}
          >
            <option value="Male">Male</option>
            <option value="Female">Female</option>
            <option value="Other">Other</option>
          </select>
        </div>

        <div className={styles.statsPreview}>
          <div className={styles.statsTitle}>Starting Stats</div>
          <div className={styles.statsList}>
            ⚔️ Might: 10
            <br />
            🛡️ Defense: 10
            <br />
            🧙 Magic: 10
            <br />
            🔮 Resistance: 10
            <br />
            💨 Agility: 10
            <br />
            ❤️ Health: 100/100
            <br />
            ⚡ Mana: 50/50
            <br />
            🗡️ Adventures: 5/5
          </div>
        </div>

        <LoadingButton
          type="submit"
          loading={loading}
          className={styles.button}
        >
          Begin Adventure
        </LoadingButton>

        {error && <div className={styles.errorMessage}>{error}</div>}
      </form>
    </div>
  );
};
