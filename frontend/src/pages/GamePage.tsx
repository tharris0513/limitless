import React, { useState, useEffect } from 'react';
import type { Player, Location } from '../types/game';
import GameAPI from '../services/api';
import { Swords, MapPin } from 'lucide-react';
import styles from './GamePage.module.css';

interface GamePageProps {
  player: Player;
  onPlayerUpdate: (player: Player) => void;
}

export const GamePage: React.FC<GamePageProps> = ({ }) => {
  const [locations, setLocations] = useState<Location[]>([]);
  const [adventures] = useState<any[]>([]); // Will be used when adventure system is implemented
  const [loading, setLoading] = useState(false);

  useEffect(() => {
    loadGameData();
  }, []);

  const loadGameData = async () => {
    try {
      setLoading(true);
      const [locationsData] = await Promise.all([
        GameAPI.getLocations(),
        // GameAPI.getAdventures() - commented out since it doesn't exist yet
      ]);
      setLocations(locationsData);
      // setAdventures(adventuresData);
    } catch (error) {
      console.error('Failed to load game data:', error);
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
        <h2 className={styles.sectionTitle}>🗺️ Locations</h2>
        {loading ? (
          <div>Loading...</div>
        ) : (
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
