import React, { useState, useEffect } from 'react';
import type { Location, Character } from '../types/game';
import GameAPI from '../services/api';
import { useGameState } from '../hooks/useGameState';
import { isCombatState, isChoiceState } from '../types/gameState';
import { CombatPage } from './CombatPage';
import { AdventurePage } from './AdventurePage';
import { Modal } from '../components/Modal';
import styles from './GamePage.module.css';

interface GamePageProps {
  character: Character;
  onCharacterUpdate?: (character: Character) => void;
  onCharacterDeleted?: () => void;
  onAbilitiesClick?: () => void;
}

export const GamePage: React.FC<GamePageProps> = ({ character, onCharacterUpdate, onCharacterDeleted, onAbilitiesClick }) => {
  const [locations, setLocations] = useState<Location[]>([]);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [characterExists, setCharacterExists] = useState(true);
  const [resting, setResting] = useState(false);
  const [showRestModal, setShowRestModal] = useState(false);
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

  const handleRest = async () => {
    if (resting) return;
    
    // Check if already at full health and mana before making API call
    if (character.health >= character.maxHealth && character.mana >= character.maxMana) {
      setShowRestModal(true);
      return;
    }
    
    setResting(true);
    try {
      const updatedCharacter = await GameAPI.restCharacter(character.id);
      
      // Notify parent component of character update
      if (onCharacterUpdate) {
        onCharacterUpdate(updatedCharacter);
      }
      
      setError(null);
    } catch (error: any) {
      console.error('Failed to rest:', error);
      // Backend returns ApiError structure: { error, message, status_code, timestamp, details }
      const errorMessage = error.response?.data?.message || error.message || 'Failed to rest. Please try again.';
      setError(errorMessage);
    } finally {
      setResting(false);
    }
  };

  const handleLocationVisit = async (locationId: string) => {
    // Prevent visiting locations if character has 0 HP
    if (character.health <= 0) {
      setError('Your character has 0 HP. You must rest before adventuring.');
      return;
    }

    try {
      setLoading(true);
      setError(null);
      
      const response = await GameAPI.visitLocation(locationId, character.id);
      
      if (response.encounterType === 'combat') {
        // Fetch creature details and start combat
        const creature = await GameAPI.adminGetCreature(response.encounterId);
        
        const combatState = {
          inCombat: true as const,
          status: 'started' as const,
          adventureId: locationId,
          adventureName: locations.find(l => l.id === locationId)?.name || 'Unknown Location',
          turnNumber: 1,
          playerHealth: character.health,
          playerMana: character.mana,
          enemy: {
            id: creature.id,
            name: creature.name,
            introductionText: creature.introductionText,
            health: creature.health ?? creature.maxHealth,
            maxHealth: creature.maxHealth,
            level: creature.level,
            experienceReward: creature.experienceReward,
            attackDescription: creature.attackDescription,
            might: creature.might,
            defense: creature.defense,
            magic: creature.magic,
            resistance: creature.resistance,
            agility: creature.agility,
          },
          combatLog: [],
        };
        
        // Save combat state to backend first, then update frontend
        await GameAPI.saveGameState(character.id, combatState);
        
        // Refresh character to get the saved game_state
        const updatedChar = await GameAPI.getCharacter(character.id);
        if (onCharacterUpdate) {
          onCharacterUpdate(updatedChar);
        }
        
        // Now set frontend state
        setGameState(combatState);
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
        {/* Action Buttons Section */}
        <div className={styles.actionsSection}>
          <button 
            className={styles.actionButton}
            onClick={handleRest}
            disabled={resting || character.adventures <= 0}
            title={character.adventures <= 0 ? "No adventures remaining" : "Restore HP and MP (costs 1 adventure)"}
          >
            🛌 {resting ? 'Resting...' : 'Rest'}
          </button>
          <button
            className={styles.actionButton}
            onClick={onAbilitiesClick}
            title="View your abilities"
          >
            ✨ Abilities
          </button>
          <button 
            className={`${styles.actionButton} ${styles.disabledButton}`}
            disabled
            title="Coming soon"
          >
            🎒 Items
          </button>
          <button 
            className={`${styles.actionButton} ${styles.disabledButton}`}
            disabled
            title="Coming soon"
          >
            👤 Character
          </button>
        </div>

        <h2 className={styles.sectionTitle}>🗺️ Locations</h2>
        {error && <div className={styles.error}>{error}</div>}
        {loading ? (
          <div>Loading locations...</div>
        ) : locations.length > 0 ? (
          locations.map((location) => (
            <div
              key={location.id}
              className={`${styles.locationCard} ${character.health <= 0 ? styles.disabledLocation : ''}`}
              onClick={() => character.health > 0 && handleLocationVisit(location.id)}
              style={{
                cursor: character.health <= 0 ? 'not-allowed' : 'pointer',
                opacity: character.health <= 0 ? 0.5 : 1,
              }}
              title={character.health <= 0 ? 'You must rest before adventuring (0 HP)' : ''}
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

      <Modal
        isOpen={showRestModal}
        onClose={() => setShowRestModal(false)}
        title="🛌 Cannot Rest"
      >
        <p>Your character is already at full health and mana. You don't need to rest right now!</p>
      </Modal>
    </div>
  );
};
