import React, { useState, useEffect } from 'react';
import type { Location, Character } from '../types/game';
import GameAPI from '../services/api';
import { useGameState } from '../hooks/useGameState';
import { isCombatState, isChoiceState } from '../types/gameState';
import { CombatPage } from './CombatPage';
import { AdventurePage } from './AdventurePage';
import styles from './GamePage.module.css';

interface GamePageProps {
  character: Character;
  onCharacterUpdate?: (character: Character) => void;
  onCharacterDeleted?: () => void;
}

export const GamePage: React.FC<GamePageProps> = ({ character, onCharacterUpdate, onCharacterDeleted }) => {
  const [locations, setLocations] = useState<Location[]>([]);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [characterExists, setCharacterExists] = useState(true);
  const { gameState, setGameState } = useGameState();

  // Validate character exists before rendering
  useEffect(() => {
    const validateCharacterExists = async () => {
      try {
        // Try to fetch the character to verify it still exists
        await GameAPI.getCharacter(character.id);
        setCharacterExists(true);
      } catch (err) {
        // Character doesn't exist (likely deleted from admin panel)
        console.warn('Character no longer exists:', character.id);
        setCharacterExists(false);
        // Clear selected character to show character selection on main route
        onCharacterDeleted?.();
      }
    };

    validateCharacterExists();
  }, [character.id]);

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
            introductionText: creature.introductionText,
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

  // Render combat or adventure page if in that state
  if (isCombatState(gameState)) {
    return <CombatPage character={character} onCharacterUpdate={onCharacterUpdate} />;
  }

  if (isChoiceState(gameState)) {
    return <AdventurePage character={character} />;
  }

  // Prevent rendering game page if character doesn't exist
  if (!characterExists) {
    return null;
  }

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
