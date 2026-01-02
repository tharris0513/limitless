import React, { useState, useEffect } from 'react';
import type { Location, Character } from '../types/game';
import GameAPI from '../services/api';
import styles from './GamePage.module.css';

interface GamePageProps {
  character: Character;
  onCharacterUpdate?: (character: Character) => void;
}

export const GamePage: React.FC<GamePageProps> = () => {
  const [locations, setLocations] = useState<Location[]>([]);
  const [_adventures] = useState<any[]>([]); // Will be used when adventure system is implemented
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

  const handleLocationVisit = (locationId: string) => {
    // Handle location visits
    console.log('Visiting location:', locationId);
  };

  return (
    <div className={styles.gameContainer}>
      <div className={styles.section}>
        <h2 className={styles.sectionTitle}>🗺️ Locations</h2>
        {error && <div className={styles.error}>{error}</div>}
        {loading ? (
          <div>Loading locations...</div>
        ) : locations.length > 0 ? (
          locations.map((location) => (
            <div
              key={location.id}
              className={styles.locationCard}
              onClick={() => handleLocationVisit(location.id)}
            >
              <div className={styles.tierLabel}>TIER {location.tier}</div>
              <div className={styles.locationTitle}>
                {location.name}
              </div>
              <div className={styles.locationDesc}>{location.description}</div>
            </div>
          ))
        ) : (
          <div className={styles.noContent}>No locations available yet.</div>
        )}
      </div>
    </div>
  );
};
