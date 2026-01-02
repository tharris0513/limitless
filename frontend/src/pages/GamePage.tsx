import React, { useState, useEffect } from 'react';
import { useNavigate } from 'react-router-dom';
import type { Location, Character } from '../types/game';
import GameAPI from '../services/api';
import { useGameState } from '../hooks/useGameState';
import { isCombatState, isChoiceState } from '../types/gameState';
import styles from './GamePage.module.css';

interface GamePageProps {
  character: Character;
  onCharacterUpdate?: (character: Character) => void;
}

export const GamePage: React.FC<GamePageProps> = ({ character }) => {
  const [locations, setLocations] = useState<Location[]>([]);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const { gameState, setGameState } = useGameState();
  const navigate = useNavigate();

  useEffect(() => {
    loadGameData();
  }, []);

  // Navigate to appropriate page when game state changes
  useEffect(() => {
    if (isCombatState(gameState)) {
      navigate('/combat');
    } else if (isChoiceState(gameState)) {
      navigate('/adventure');
    }
  }, [gameState, navigate]);

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

  const handleLocationVisit = async (locationId: string) => {
    try {
      setLoading(true);
      setError(null);
      
      const response = await GameAPI.visitLocation(locationId);
      
      if (response.encounterType === 'combat') {
        // Fetch creature details and start combat
        const creature = await GameAPI.adminGetCreature(response.encounterId);
        
        setGameState({
          inCombat: true,
          adventureId: locationId,
          adventureName: locations.find(l => l.id === locationId)?.name || 'Unknown Location',
          turnNumber: 1,
          playerHealth: character.health,
          playerMana: character.mana,
          enemy: {
            id: creature.id,
            name: creature.name,
            health: creature.health,
            maxHealth: creature.health,
            level: creature.level,
            stats: {
              might: creature.might,
              defense: creature.defense,
              magic: creature.magic,
              resistance: creature.resistance,
              agility: creature.agility,
            },
          },
          combatLog: [],
        });
      } else if (response.encounterType === 'adventure') {
        // Start adventure - TODO: Fetch actual adventure details
        setGameState({
          inChoice: true,
          adventureId: response.encounterId,
          adventureName: 'Adventure', // Will be fetched from adventure API
          sceneId: 'scene-1',
          sceneName: 'The Beginning',
          sceneDescription: 'You embark on a new adventure...',
          choices: [
            {
              id: 'choice-1',
              text: 'Continue',
              description: 'Proceed with the adventure',
            },
          ],
        });
      }
    } catch (error) {
      console.error('Failed to visit location:', error);
      setError('Failed to visit location. Please try again.');
    } finally {
      setLoading(false);
    }
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
