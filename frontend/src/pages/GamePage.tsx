import React, { useState, useEffect } from 'react';
import type { Location, Character } from '../types/game';
import GameAPI from '../services/api';
import { Swords, MapPin } from 'lucide-react';
import styles from './GamePage.module.css';

interface GamePageProps {
  character: Character;
  onCharacterUpdate?: (character: Character) => void;
}

export const GamePage: React.FC<GamePageProps> = ({ character }) => {
  const [locations, setLocations] = useState<Location[]>([]);
  const [adventures] = useState<any[]>([]); // Will be used when adventure system is implemented
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    loadGameData();
  }, []);

  const loadGameData = async () => {
    try {
      setLoading(true);
      setError(null);
      const locationsData = await GameAPI.getLocations();
      setLocations(locationsData);
    } catch (error) {
      console.error('Failed to load game data:', error);
      setError('Failed to load game locations. Please try again.');
    } finally {
      setLoading(false);
    }
  };

  const handleAdventure = async (adventureId: string) => {
    try {
      setLoading(true);
      // const result = await GameAPI.completeAdventure(adventureId);
      // onPlayerUpdate(result.player);
      // alert(`Adventure completed! You gained ${result.rewards.experience} XP`);
      console.log('Adventure:', adventureId);
      alert('Adventure system coming soon!');
    } catch (error) {
      console.error('Adventure failed:', error);
      alert('Adventure failed!');
    } finally {
      setLoading(false);
    }
  };

  const handleLocationVisit = (locationId: string) => {
    // Handle location visits
    console.log('Visiting location:', locationId);
  };

  return (
    <div className={styles.gameContainer}>
      <div className={styles.section}>
        <h2 className={styles.sectionTitle}>⚔️ {character.name}</h2>
        <div className={styles.characterInfo}>
          <p>Level {character.level} | {character.location}</p>
          <p>Adventures: {character.stats.adventures}/{character.stats.maxAdventures}</p>
          <p>HP: {character.health}/{character.maxHealth} | MP: {character.mana}/{character.maxMana}</p>
        </div>
      </div>
      
      <div className={styles.section}>
        <h2 className={styles.sectionTitle}>🗺️ Locations</h2>
        {error && <div className={styles.error}>{error}</div>}
        {loading ? (
          <div>Loading locations...</div>
        ) : locations.length > 0 ? (
          locations.map(location => (
            <div
              key={location.id}
              className={styles.locationCard}
              onClick={() => handleLocationVisit(location.id)}
            >
              <div className={styles.locationName}>
                <MapPin size={16} /> {location.name}
              </div>
              <div className={styles.locationDesc}>{location.description}</div>
            </div>
          ))
        ) : (
          <div className={styles.noContent}>No locations available yet.</div>
        )}
      </div>

      <div className={styles.section}>
        <h2 className={styles.sectionTitle}>⚔️ Adventures</h2>
        {loading ? (
          <div>Loading...</div>
        ) : (
          adventures.map(adventure => (
            <div
              key={adventure.id}
              className={styles.adventureCard}
              onClick={() => handleAdventure(adventure.id)}
            >
              <div className={styles.adventureName}>
                <Swords size={16} /> {adventure.name}
              </div>
              <div className={styles.adventureDesc}>
                {adventure.description}
              </div>
              <div className={styles.adventureReward}>
                Reward: {adventure.rewards?.experience || 0} XP
              </div>
            </div>
          ))
        )}
      </div>
    </div>
  );
};
