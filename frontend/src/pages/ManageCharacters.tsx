import React, { useEffect, useState, useRef } from 'react';
import { createPortal } from 'react-dom';
import {
  ArrowLeft,
  User as UserIcon,
  ChevronDown,
  Edit,
  Trash2,
  Sword,
  Swords,
} from 'lucide-react';
import axios from 'axios';
import styles from './ManageCharacters.module.css';

interface Ability {
  id: string;
  name: string;
  description?: string;
  ability_type?: string;
  mana_cost?: number;
  cooldown?: number;
  damage_formula?: string;
  heal_formula?: string;
  effect_formula?: string;
}

interface Character {
  id: string;
  userId: string;
  name: string;
  classId: string;
  level: number;
  experience: number;
  experienceToNext: number;
  stats: {
    health: number;
    maxHealth: number;
    mana: number;
    maxMana: number;
    might: number;
    defense: number;
    magic: number;
    resistance: number;
    agility: number;
  };
  createdAt: string;
  lastPlayed: string;
  adventures: number;
  abilities?: Ability[];
}

interface AbilitySummary {
  id: string;
  name: string;
  description?: string;
  ability_type?: string;
  mana_cost?: number;
  cooldown?: number;
  damage_formula?: string;
  heal_formula?: string;
  effect_formula?: string;
}

interface User {
  id: string;
  username?: string;
  discordName: string;
}

interface ManageCharactersProps {
  onBack: () => void;
  onLogout?: () => void;
}

export const ManageCharacters: React.FC<ManageCharactersProps> = ({
  onBack,
}) => {
  const [characters, setCharacters] = useState<Character[]>([]);
  const [users, setUsers] = useState<Map<string, User>>(new Map());
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);
  const [openDropdown, setOpenDropdown] = useState<string | null>(null);
  const [actionLoading, setActionLoading] = useState(false);
  const [dropdownPosition, setDropdownPosition] = useState<{
    top: number;
    left: number;
  } | null>(null);
  const [detailModalId, setDetailModalId] = useState<string | null>(null);
  const [selectedCharacter, setSelectedCharacter] = useState<
    Character | null
  >(null);
  const [detailLoading, setDetailLoading] = useState(false);
  const [detailError, setDetailError] = useState<string | null>(null);
  const buttonRefs = useRef<Map<string, HTMLButtonElement>>(new Map());

  useEffect(() => {
    const fetchData = async () => {
      try {
        setLoading(true);
        setError(null);
        const token = localStorage.getItem('authToken');
        const API_BASE_URL =
          import.meta.env.VITE_API_URL || 'http://localhost:8080/api';

        // First, get all users
        const usersResponse = await axios.get(`${API_BASE_URL}/admin/users`, {
          headers: {
            Authorization: `Bearer ${token}`,
          },
        });

        const usersMap = new Map<string, User>();
        // eslint-disable-next-line @typescript-eslint/no-explicit-any
        usersResponse.data.forEach((user: any) => {
          usersMap.set(user.id, {
            id: user.id,
            username: user.username,
            discordName: user.discord_name,
          });
        });
        setUsers(usersMap);

        // Get all characters
        const charactersResponse = await axios.get(
          `${API_BASE_URL}/admin/characters`,
          {
            headers: {
              Authorization: `Bearer ${token}`,
            },
          }
        );
        const transformedCharacters = charactersResponse.data.map(
        // eslint-disable-next-line @typescript-eslint/no-explicit-any
          (char: any) => ({
            id: char.id,
            userId: char.userId || char.user_id,
            name: char.name,
            classId: char.classId || char.class_id,
            level: char.level,
            experience: char.experience,
            experienceToNext: char.experienceToNext || char.experience_to_next,
            stats: char.stats,
            createdAt: char.createdAt || char.created_at,
            lastPlayed: char.lastPlayed || char.last_played,
            adventures: char.stats?.adventures || 0,
          })
        );

        setCharacters(transformedCharacters);
      } catch (err) {
        console.error('Failed to fetch data:', err);
        if (axios.isAxiosError(err) && err.response?.status === 403) {
          setError('Access denied - Admin privileges required');
        } else {
          setError('Failed to load characters');
        }
      } finally {
        setLoading(false);
      }
    };

    fetchData();
  }, []);

  // Handle dropdown toggle with positioning
  const handleDropdownToggle = (
    characterId: string,
    event: React.MouseEvent<HTMLButtonElement>
  ) => {
    if (openDropdown === characterId) {
      setOpenDropdown(null);
      setDropdownPosition(null);
    } else {
      const button = event.currentTarget;
      const rect = button.getBoundingClientRect();
      setDropdownPosition({
        top: rect.bottom + window.scrollY + 8,
        left: rect.right + window.scrollX - 180,
      });
      setOpenDropdown(characterId);
      buttonRefs.current.set(characterId, button);
    }
  };

  // Close dropdown when clicking outside
  useEffect(() => {
    const handleClickOutside = (event: MouseEvent) => {
      if (openDropdown) {
        const button = buttonRefs.current.get(openDropdown);
        const dropdown = document.getElementById('character-actions-dropdown');
        if (
          button &&
          dropdown &&
          !button.contains(event.target as Node) &&
          !dropdown.contains(event.target as Node)
        ) {
          setOpenDropdown(null);
          setDropdownPosition(null);
        }
      }
    };

    document.addEventListener('mousedown', handleClickOutside);
    return () => document.removeEventListener('mousedown', handleClickOutside);
  }, [openDropdown]);

  const handleEditCharacter = async (character: Character) => {
    const newName = prompt(
      `Enter new name for ${character.name}:`,
      character.name
    );
    if (!newName || newName === character.name) return;

    try {
      setActionLoading(true);
      const token = localStorage.getItem('authToken');
      const API_BASE_URL =
        import.meta.env.VITE_API_URL || 'http://localhost:8080/api';

      await axios.patch(
        `${API_BASE_URL}/admin/characters/${character.id}`,
        { name: newName },
        {
          headers: {
            Authorization: `Bearer ${token}`,
          },
        }
      );

      setCharacters(
        characters.map(c =>
          c.id === character.id ? { ...c, name: newName } : c
        )
      );
      setOpenDropdown(null);
    } catch (err) {
      console.error('Failed to update character:', err);
      alert('Failed to update character name');
    } finally {
      setActionLoading(false);
    }
  };

  const handleSetAdventures = async (character: Character) => {
    const adventuresInput = prompt(
      `Enter number of adventures for ${character.name} (current: ${character.adventures}):`,
      character.adventures.toString()
    );

    if (
      !adventuresInput ||
      adventuresInput === character.adventures.toString()
    )
      return;

    const adventures = parseInt(adventuresInput, 10);
    if (isNaN(adventures) || adventures < 0) {
      alert('Please enter a valid positive number');
      return;
    }

    try {
      setActionLoading(true);
      const token = localStorage.getItem('authToken');
      const API_BASE_URL =
        import.meta.env.VITE_API_URL || 'http://localhost:8080/api';

      await axios.patch(
        `${API_BASE_URL}/admin/characters/${character.id}/adventures`,
        { adventures },
        {
          headers: {
            Authorization: `Bearer ${token}`,
          },
        }
      );

      setCharacters(
        characters.map(c =>
          c.id === character.id
            ? { ...c, stats: { ...c.stats, adventures }, adventures }
            : c
        )
      );
      setOpenDropdown(null);
    } catch (err) {
      console.error('Failed to update adventures:', err);
      alert('Failed to update adventures count');
    } finally {
      setActionLoading(false);
    }
  };

  const handleDeleteCharacter = async (character: Character) => {
    const user = users.get(character.userId);
    const userName = user?.username || user?.discordName || 'Unknown User';

    if (
      !confirm(
        `Are you sure you want to delete character "${character.name}" (owned by ${userName})? This action cannot be undone.`
      )
    ) {
      return;
    }

    try {
      setActionLoading(true);
      const token = localStorage.getItem('authToken');
      const API_BASE_URL =
        import.meta.env.VITE_API_URL || 'http://localhost:8080/api';

      await axios.delete(`${API_BASE_URL}/admin/characters/${character.id}`, {
        headers: {
          Authorization: `Bearer ${token}`,
        },
      });

      setCharacters(characters.filter(c => c.id !== character.id));
      setOpenDropdown(null);
    } catch (err) {
      console.error('Failed to delete character:', err);
      alert('Failed to delete character');
    } finally {
      setActionLoading(false);
    }
  };

  const handleExamineCharacter = async (character: Character) => {
    setDetailModalId(character.id);
    setDetailLoading(true);
    setDetailError(null);
    setSelectedCharacter(character);
    setOpenDropdown(null);

    try {
      const token = localStorage.getItem('authToken');
      const API_BASE_URL =
        import.meta.env.VITE_API_URL || 'http://localhost:8080/api';

      const response = await axios.get(
        `${API_BASE_URL}/admin/characters/${character.id}`,
        {
          headers: {
            Authorization: `Bearer ${token}`,
          },
        }
      );

      const data = response.data;
      const abilities: AbilitySummary[] =
        data.abilities || data.characterAbilities || data.ability || [];

      setSelectedCharacter({
        ...character,
        ...data,
        abilities,
      });
    } catch (err) {
      console.error('Failed to load character details:', err);
      setDetailError('Failed to load character details');
    } finally {
      setDetailLoading(false);
    }
  };

  const getUserDisplay = (userId: string): string => {
    const user = users.get(userId);
    return user?.username || user?.discordName || 'Unknown User';
  };

  return (
    <div className={styles.manageCharactersContainer}>
      <div className={styles.header}>
        <button className={styles.backButton} onClick={onBack}>
          <ArrowLeft size={20} />
          <span>Back</span>
        </button>
        <h1 className={styles.title}>⚔️ Manage Characters</h1>
      </div>

      <div className={styles.content}>
        {loading && (
          <div className={styles.loading}>
            <div className={styles.loadingText}>Loading characters...</div>
          </div>
        )}

        {error && (
          <div className={styles.error}>
            <span className={styles.errorIcon}>⚠️</span>
            <span>{error}</span>
          </div>
        )}

        {!loading && !error && (
          <div className={styles.tableContainer}>
            <div className={styles.tableHeader}>
              <div className={styles.statsBar}>
                Total Characters:{' '}
                <span className={styles.highlight}>{characters.length}</span>
                {' | '}
                Unique Users:{' '}
                <span className={styles.highlight}>{users.size}</span>
              </div>
            </div>

            <table className={styles.characterTable}>
              <thead>
                <tr>
                  <th>Character Name</th>
                  <th>Owner</th>
                  <th>Class</th>
                  <th>Level</th>
                  <th>Last Played</th>
                  <th className={styles.actionsHeader}>Actions</th>
                </tr>
              </thead>
              <tbody>
                {characters.map(character => (
                  <tr key={character.id}>
                    <td>
                      <div className={styles.nameCell}>
                        <Sword size={16} />
                        {character.name}
                      </div>
                    </td>
                    <td>
                      <div className={styles.nameCell}>
                        <UserIcon size={16} />
                        {getUserDisplay(character.userId)}
                      </div>
                    </td>
                    <td className={styles.classCell}>{character.classId}</td>
                    <td className={styles.levelCell}>
                      <span className={styles.levelBadge}>
                        Lv {character.level}
                      </span>
                    </td>
                    <td className={styles.dateCell}>
                      {character.lastPlayed
                        ? new Date(character.lastPlayed).toLocaleDateString(
                            'en-US',
                            {
                              year: 'numeric',
                              month: 'short',
                              day: 'numeric',
                            }
                          )
                        : 'Never'}
                    </td>
                    <td className={styles.actionsCell}>
                      <div className={styles.actionsDropdown}>
                        <button
                          className={styles.actionsButton}
                          onClick={e => handleDropdownToggle(character.id, e)}
                          disabled={actionLoading}
                        >
                          Actions <ChevronDown size={14} />
                        </button>
                      </div>
                    </td>
                  </tr>
                ))}
              </tbody>
            </table>

            {characters.length === 0 && (
              <div className={styles.emptyState}>
                No characters found in the database.
              </div>
            )}
          </div>
        )}
      </div>

      {/* Render dropdown using portal */}
      {openDropdown &&
        dropdownPosition &&
        createPortal(
          <div
            id="character-actions-dropdown"
            className={styles.dropdownMenu}
            style={{
              position: 'fixed',
              top: `${dropdownPosition.top}px`,
              left: `${dropdownPosition.left}px`,
            }}
          >
            <button
              className={styles.dropdownItem}
              onClick={() => {
                const character = characters.find(c => c.id === openDropdown);
                if (character) handleExamineCharacter(character);
              }}
              disabled={actionLoading}
            >
              <Swords size={14} />
              Examine
            </button>
            <button
              className={styles.dropdownItem}
              onClick={() => {
                const character = characters.find(c => c.id === openDropdown);
                if (character) handleEditCharacter(character);
              }}
              disabled={actionLoading}
            >
              <Edit size={14} />
              Change Name
            </button>
            <button
              className={styles.dropdownItem}
              onClick={() => {
                const character = characters.find(c => c.id === openDropdown);
                if (character) handleSetAdventures(character);
              }}
              disabled={actionLoading}
            >
              <Swords size={14} />
              Set Adventures
            </button>
            <button
              className={`${styles.dropdownItem} ${styles.deleteItem}`}
              onClick={() => {
                const character = characters.find(c => c.id === openDropdown);
                if (character) handleDeleteCharacter(character);
              }}
              disabled={actionLoading}
            >
              <Trash2 size={14} />
              Delete Character
            </button>
          </div>,
          document.body
        )}

      {detailModalId && (
        <div className={styles.modalOverlay}>
          <div className={styles.modalContent}>
            <div className={styles.modalHeader}>
              <div>
                <div className={styles.modalTitle}>Character Details</div>
                <div className={styles.modalSubtitle}>
                  {selectedCharacter?.name || 'Loading...'}
                </div>
              </div>
              <button
                className={styles.closeButton}
                onClick={() => {
                  setDetailModalId(null);
                  setSelectedCharacter(null);
                  setDetailError(null);
                }}
              >
                ×
              </button>
            </div>

            {detailLoading && (
              <div className={styles.modalLoading}>Loading details...</div>
            )}

            {detailError && !detailLoading && (
              <div className={styles.modalError}>{detailError}</div>
            )}

            {selectedCharacter && !detailLoading && (
              <div className={styles.modalBody}>
                <div className={styles.detailGrid}>
                  <div>
                    <div className={styles.detailLabel}>Owner</div>
                    <div className={styles.detailValue}>
                      {getUserDisplay(selectedCharacter.userId)}
                    </div>
                  </div>
                  <div>
                    <div className={styles.detailLabel}>Class</div>
                    <div className={styles.detailValue}>
                      {selectedCharacter.classId}
                    </div>
                  </div>
                  <div>
                    <div className={styles.detailLabel}>Level</div>
                    <div className={styles.detailValue}>
                      {selectedCharacter.level}
                    </div>
                  </div>
                  <div>
                    <div className={styles.detailLabel}>Adventures</div>
                    <div className={styles.detailValue}>
                      {selectedCharacter.adventures}
                    </div>
                  </div>
                  <div>
                    <div className={styles.detailLabel}>Health</div>
                    <div className={styles.detailValue}>
                      {selectedCharacter.stats?.health}/
                      {selectedCharacter.stats?.maxHealth}
                    </div>
                  </div>
                  <div>
                    <div className={styles.detailLabel}>Mana</div>
                    <div className={styles.detailValue}>
                      {selectedCharacter.stats?.mana}/
                      {selectedCharacter.stats?.maxMana}
                    </div>
                  </div>
                  <div>
                    <div className={styles.detailLabel}>Might</div>
                    <div className={styles.detailValue}>
                      {selectedCharacter.stats?.might}
                    </div>
                  </div>
                  <div>
                    <div className={styles.detailLabel}>Defense</div>
                    <div className={styles.detailValue}>
                      {selectedCharacter.stats?.defense}
                    </div>
                  </div>
                  <div>
                    <div className={styles.detailLabel}>Magic</div>
                    <div className={styles.detailValue}>
                      {selectedCharacter.stats?.magic}
                    </div>
                  </div>
                  <div>
                    <div className={styles.detailLabel}>Resistance</div>
                    <div className={styles.detailValue}>
                      {selectedCharacter.stats?.resistance}
                    </div>
                  </div>
                  <div>
                    <div className={styles.detailLabel}>Agility</div>
                    <div className={styles.detailValue}>
                      {selectedCharacter.stats?.agility}
                    </div>
                  </div>
                  <div>
                    <div className={styles.detailLabel}>Experience</div>
                    <div className={styles.detailValue}>
                      {selectedCharacter.experience}
                    </div>
                  </div>
                  <div>
                    <div className={styles.detailLabel}>Experience to Next</div>
                    <div className={styles.detailValue}>
                      {selectedCharacter.experienceToNext}
                    </div>
                  </div>
                  <div>
                    <div className={styles.detailLabel}>Created</div>
                    <div className={styles.detailValue}>
                      {selectedCharacter.createdAt
                        ? new Date(
                            selectedCharacter.createdAt
                          ).toLocaleDateString('en-US', {
                            year: 'numeric',
                            month: 'short',
                            day: 'numeric',
                          })
                        : 'Unknown'}
                    </div>
                  </div>
                  <div>
                    <div className={styles.detailLabel}>Last Played</div>
                    <div className={styles.detailValue}>
                      {selectedCharacter.lastPlayed
                        ? new Date(
                            selectedCharacter.lastPlayed
                          ).toLocaleDateString('en-US', {
                            year: 'numeric',
                            month: 'short',
                            day: 'numeric',
                          })
                        : 'Never'}
                    </div>
                  </div>
                </div>

                <div className={styles.abilitiesSection}>
                  <div className={styles.sectionTitle}>Abilities</div>
                  {selectedCharacter.abilities &&
                  selectedCharacter.abilities.length > 0 ? (
                    <div className={styles.abilityList}>
                      {selectedCharacter.abilities.map(ability => (
                        <div className={styles.abilityCard} key={ability.id}>
                          <div className={styles.abilityHeader}>
                            <div className={styles.abilityName}>
                              {ability.name}
                            </div>
                            <div className={styles.abilityMeta}>
                              {ability.ability_type && (
                                <span>{ability.ability_type}</span>
                              )}
                              {typeof ability.mana_cost === 'number' && (
                                <span>Mana: {ability.mana_cost}</span>
                              )}
                              {typeof ability.cooldown === 'number' && (
                                <span>CD: {ability.cooldown}</span>
                              )}
                            </div>
                          </div>
                          {ability.description && (
                            <div className={styles.abilityDescription}>
                              {ability.description}
                            </div>
                          )}
                          <div className={styles.formulasGrid}>
                            {ability.damage_formula && (
                              <div>
                                <div className={styles.detailLabel}>Damage</div>
                                <div className={styles.formulaValue}>
                                  {ability.damage_formula}
                                </div>
                              </div>
                            )}
                            {ability.heal_formula && (
                              <div>
                                <div className={styles.detailLabel}>Heal</div>
                                <div className={styles.formulaValue}>
                                  {ability.heal_formula}
                                </div>
                              </div>
                            )}
                            {ability.effect_formula && (
                              <div>
                                <div className={styles.detailLabel}>Effect</div>
                                <div className={styles.formulaValue}>
                                  {ability.effect_formula}
                                </div>
                              </div>
                            )}
                          </div>
                        </div>
                      ))}
                    </div>
                  ) : (
                    <div className={styles.emptyAbilities}>
                      No abilities found for this character.
                    </div>
                  )}
                </div>
              </div>
            )}
          </div>
        </div>
      )}
    </div>
  );
};
