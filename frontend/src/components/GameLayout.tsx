import React, { useMemo } from 'react';
import { type Character, type User } from '../types/game';
import { Heart, Zap, Star, Settings, Shield, Swords, Dumbbell, ShieldCheck } from 'lucide-react';
import { ChatPanel } from './ChatPanel';
import styles from './GameLayout.module.css';

interface GameLayoutProps {
  character: Character | null;
  user?: User | null;
  children: React.ReactNode;
  onLogout?: () => void;
  onSettingsClick?: () => void;
  isAdmin?: boolean;
  onAdminClick?: () => void;
  hideCharacterInfo?: boolean;
}

export const GameLayout: React.FC<GameLayoutProps> = ({ character, user, children, onLogout, onSettingsClick, isAdmin, onAdminClick, hideCharacterInfo }) => {
  // Use experience and experienceToNext directly from the character
  const expProgress = useMemo(() => {
    if (!character) return { current: 0, needed: 100 };
    return { 
      current: character.experience, 
      needed: character.experienceToNext 
    };
  }, [character]);

  return (
    <div className={styles.layoutContainer}>
      <header className={styles.header}>
        <div className={styles.headerContent}>
          <h1 className={styles.gameTitle}>Limitless</h1>
          
          {onLogout && (
            <div className={styles.headerButtons}>
              {isAdmin && (
                <button 
                  className={styles.adminButton} 
                  onClick={onAdminClick}
                  title="Admin Panel"
                >
                  <Shield size={18} />
                  <span>Admin</span>
                </button>
              )}
              {onSettingsClick && (
                <button 
                  className={styles.settingsButton} 
                  onClick={onSettingsClick}
                  title="Settings"
                >
                  <Settings size={20} />
                </button>
              )}
              <button className={styles.logoutButton} onClick={onLogout}>
                Logout
              </button>
            </div>
          )}
        </div>
      </header>

      <div className={styles.layoutBody}>
        {character && !hideCharacterInfo && (
          <aside className={styles.sidebar}>
            <div className={styles.sidebarContent}>
              <div className={styles.characterHeader}>
                <h2 className={styles.characterName}>{character.name}</h2>
                <div className={styles.characterLevel}>Level {character.level}</div>
                <div className={styles.characterClass}>{character.classId}</div>
                
                <div className={styles.experienceSection}>
                  <div className={styles.experienceLabel}>
                    <span>XP: {expProgress.current.toLocaleString()} / {expProgress.needed.toLocaleString()}</span>
                  </div>
                  <div className={styles.experienceBar}>
                    <div 
                      className={styles.experienceProgress}
                      style={{ width: `${(expProgress.current / expProgress.needed) * 100}%` }}
                    />
                  </div>
                </div>
              </div>

              <div className={styles.characterStats}>
                <div className={styles.statItem}>
                  <Heart size={16} className={styles.statIcon} />
                  <div className={styles.statInfo}>
                    <span className={styles.statValue}>{character.health}/{character.calculatedStats?.maxHealth ?? character.maxHealth}</span>
                    <span className={styles.statLabel}>Health</span>
                  </div>
                </div>
                <div className={styles.statItem}>
                  <Star size={16} className={styles.statIcon} />
                  <div className={styles.statInfo}>
                    <span className={styles.statValue}>{character.mana}/{character.calculatedStats?.maxMana ?? character.maxMana}</span>
                    <span className={styles.statLabel}>Mana</span>
                  </div>
                </div>
                <div className={styles.statItem}>
                  <Dumbbell size={16} className={styles.statIcon} />
                  <div className={styles.statInfo}>
                    <span className={styles.statValue}>{character.calculatedStats?.might ?? character.might ?? 0}</span>
                    <span className={styles.statLabel}>Might</span>
                  </div>
                </div>
                <div className={styles.statItem}>
                  <Shield size={16} className={styles.statIcon} />
                  <div className={styles.statInfo}>
                    <span className={styles.statValue}>{character.calculatedStats?.defense ?? character.defense ?? 0}</span>
                    <span className={styles.statLabel}>Defense</span>
                  </div>
                </div>
                <div className={styles.statItem}>
                  <Star size={16} className={styles.statIcon} />
                  <div className={styles.statInfo}>
                    <span className={styles.statValue}>{character.calculatedStats?.magic ?? character.magic ?? 0}</span>
                    <span className={styles.statLabel}>Magic</span>
                  </div>
                </div>
                <div className={styles.statItem}>
                  <ShieldCheck size={16} className={styles.statIcon} />
                  <div className={styles.statInfo}>
                    <span className={styles.statValue}>{character.calculatedStats?.resistance ?? character.resistance ?? 0}</span>
                    <span className={styles.statLabel}>Resistance</span>
                  </div>
                </div>
                <div className={styles.statItem}>
                  <Zap size={16} className={styles.statIcon} />
                  <div className={styles.statInfo}>
                    <span className={styles.statValue}>{character.calculatedStats?.agility ?? character.agility ?? 0}</span>
                    <span className={styles.statLabel}>Agility</span>
                  </div>
                </div>
                <div className={styles.statItem}>
                  <Swords size={16} className={styles.statIcon} />
                  <div className={styles.statInfo}>
                    <span className={styles.statValue}>{character.adventures || 0}</span>
                    <span className={styles.statLabel}>Adventures</span>
                  </div>
                </div>
              </div>

              {character.activeBuffs && character.activeBuffs.length > 0 && (
                <div className={styles.activeBuffsSection}>
                  <h3 className={styles.activeBuffsTitle}>Active Buffs</h3>
                  <div className={styles.activeBuffsList}>
                    {character.activeBuffs.map((buff) => (
                      <div key={buff.abilityId} className={styles.buffItem}>
                        <div className={styles.buffName}>{buff.abilityName}</div>
                        <div className={styles.buffDuration}>
                          {buff.remainingAdventures} adv
                        </div>
                      </div>
                    ))}
                  </div>
                </div>
              )}
            </div>
          </aside>
        )}

        <main className={styles.mainContent}>{children}</main>

        {user && (
          <ChatPanel username={user.username || user.discordName} />
        )}
      </div>
    </div>
  );
};
