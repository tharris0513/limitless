import React, { useState, useEffect } from 'react';
import { LoadingButton } from '../components/LoadingStates';
import { useApiWithLoading } from '../hooks/useApiWithLoading';
import type { Character, User, Class } from '../types/game';
import GameAPI from '../services/api';
import { containsProfanity, getProfanityErrorMessage } from '../utils/profanityFilter';
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
  const [classes, setClasses] = useState<Class[]>([]);
  const [selectedClass, setSelectedClass] = useState<Class | null>(null);
  const [formData, setFormData] = useState({
    name: '',
    classId: '',
    gender: '',
  });
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState('');

  useEffect(() => {
    loadClasses();
  }, []);

  const loadClasses = async () => {
    try {
      const classesData = await GameAPI.getClasses();
      setClasses(classesData);
      if (classesData.length > 0) {
        setSelectedClass(classesData[0]);
        setFormData(prev => ({ ...prev, classId: classesData[0].id }));
      }
    } catch (err) {
      console.error('Failed to load classes:', err);
      setError('Failed to load character classes');
    }
  };

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

    if (formData.name.length > 30) {
      setError('Character name must be no more than 30 characters');
      return;
    }

    // Check for profanity in character name
    if (containsProfanity(formData.name)) {
      setError(getProfanityErrorMessage());
      return;
    }

    if (!formData.classId) {
      setError('Please select a character class');
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
        classId: formData.classId,
      });

      console.log('Created character:', newCharacter);
      onCharacterCreated(newCharacter);
    } catch (err: any) {
      console.error('Character creation error:', err);
      const errorMessage = err?.response?.data?.error || err?.message || 'Failed to create character. Please try again.';
      setError(errorMessage);
    } finally {
      setLoading(false);
    }
  };

  const handleChange = (
    e: React.ChangeEvent<HTMLInputElement | HTMLSelectElement>
  ) => {
    const { name, value } = e.target;
    setFormData(prev => ({
      ...prev,
      [name]: value,
    }));

    if (name === 'classId') {
      const selected = classes.find(c => c.id === value);
      setSelectedClass(selected || null);
    }
  };

  return (
    <div className={styles.characterContainer}>
      <h1 className={styles.title}>⚔️ Create Character</h1>

      <div className={styles.welcomeText}>
        Welcome, <span className={styles.highlight}>{user.username || user.discordName}</span>!<br />
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
          <label htmlFor="classId" className={styles.label}>
            Character Class
          </label>
          <select
            id="classId"
            name="classId"
            value={formData.classId}
            onChange={handleChange}
            className={styles.select}
            disabled={loading || classes.length === 0}
          >
            {classes.map(classOption => (
              <option key={classOption.id} value={classOption.id}>
                {classOption.name}
              </option>
            ))}
          </select>
          {selectedClass && (
            <p className={styles.classDescription}>{selectedClass.description}</p>
          )}
        </div>

        {selectedClass && (
          <div className={styles.statsPreview}>
            <div className={styles.statsTitle}>Starting Stats - {selectedClass.name}</div>
            <div className={styles.statsList}>
              ⚔️ Might: {selectedClass.startingMight}
              <br />
              🛡️ Defense: {selectedClass.startingDefense}
              <br />
              🧙 Magic: {selectedClass.startingMagic}
              <br />
              🔮 Resistance: {selectedClass.startingResistance}
              <br />
              💨 Agility: {selectedClass.startingAgility}
              <br />
              ❤️ Health: {selectedClass.startingHealth}/{selectedClass.startingHealth}
              <br />
              ⚡ Mana: {selectedClass.startingMana}/{selectedClass.startingMana}
              <br />
              🗡️ Adventures: 5
            </div>
          </div>
        )}

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
