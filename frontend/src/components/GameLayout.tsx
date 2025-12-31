import React from 'react';
import { type Player, type User } from '../types/game';
import { Heart, Zap, Star, Settings, Shield } from 'lucide-react';
import { ChatPanel } from './ChatPanel';
import styles from './GameLayout.module.css';

interface GameLayoutProps {
  player: Player | null;
  user?: User | null;
  children: React.ReactNode;
  onLogout?: () => void;
  onSettingsClick?: () => void;
  isAdmin?: boolean;
  onAdminClick?: () => void;
}

export const GameLayout: React.FC<GameLayoutProps> = ({ player, user, children, onLogout, onSettingsClick, isAdmin, onAdminClick }) => {
  return (
    <div className={styles.layoutContainer}>
      <header className={styles.header}>
        <div className={styles.headerContent}>
          <h1 className={styles.gameTitle}>🏰 Limitless</h1>
          
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
        {player && player.stats && (
          <aside className={styles.sidebar}>
            <div className={styles.sidebarContent}>
              <div className={styles.characterHeader}>
                <h2 className={styles.characterName}>{player.name}</h2>
                <div className={styles.characterLevel}>Level {player.level}</div>
              </div>

              <div className={styles.characterStats}>
                <div className={styles.statItem}>
                  <Heart size={16} className={styles.statIcon} />
                  <div className={styles.statInfo}>
                    <span className={styles.statValue}>{player.stats.might}</span>
                    <span className={styles.statLabel}>Might</span>
                  </div>
                </div>
                <div className={styles.statItem}>
                  <Shield size={16} className={styles.statIcon} />
                  <div className={styles.statInfo}>
                    <span className={styles.statValue}>{player.stats.defense}</span>
                    <span className={styles.statLabel}>Defense</span>
                  </div>
                </div>
                <div className={styles.statItem}>
                  <Star size={16} className={styles.statIcon} />
                  <div className={styles.statInfo}>
                    <span className={styles.statValue}>{player.stats.magic}</span>
                    <span className={styles.statLabel}>Magic</span>
                  </div>
                </div>
                <div className={styles.statItem}>
                  <Heart size={16} className={styles.statIcon} />
                  <div className={styles.statInfo}>
                    <span className={styles.statValue}>{player.stats.resistance}</span>
                    <span className={styles.statLabel}>Resistance</span>
                  </div>
                </div>
                <div className={styles.statItem}>
                  <Zap size={16} className={styles.statIcon} />
                  <div className={styles.statInfo}>
                    <span className={styles.statValue}>{player.stats.agility}</span>
                    <span className={styles.statLabel}>Agility</span>
                  </div>
                </div>
              </div>
            </div>
          </aside>
        )}

        <main className={styles.mainContent}>{children}</main>

        {player && player.stats && user && (
          <ChatPanel username={user.username || user.discordName} />
        )}
      </div>
    </div>
  );
};
