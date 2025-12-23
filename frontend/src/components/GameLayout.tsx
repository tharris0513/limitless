import React from 'react';
import { type Player } from '../types/game';
import { Heart, Zap, Star, Settings, Shield } from 'lucide-react';
import styles from './GameLayout.module.css';

interface GameLayoutProps {
  player: Player | null;
  children: React.ReactNode;
  onLogout?: () => void;
  onSettingsClick?: () => void;
  isAdmin?: boolean;
  onAdminClick?: () => void;
}

export const GameLayout: React.FC<GameLayoutProps> = ({ player, children, onLogout, onSettingsClick, isAdmin, onAdminClick }) => {
  return (
    <div className={styles.layoutContainer}>
      <header className={styles.header}>
        <div className={styles.headerContent}>
          <h1 className={styles.gameTitle}>🏰 Limitless</h1>

          {player && player.stats && (
            <div className={styles.playerStats}>
              <div className={styles.playerInfo}>
                <span className={styles.playerName}>{player.name}</span>
                <span className={styles.playerLevel}>Level {player.level}</span>
              </div>

              <div className={styles.stats}>
                <div className={styles.stat}>
                  <Heart size={16} className={styles.statIcon} />
                  <span>{player.stats.might}</span>
                  <span className={styles.statLabel}>Might</span>
                </div>
                <div className={styles.stat}>
                  <Zap size={16} className={styles.statIcon} />
                  <span>{player.stats.defense}</span>
                  <span className={styles.statLabel}>Defense</span>
                </div>
                <div className={styles.stat}>
                  <Star size={16} className={styles.statIcon} />
                  <span>{player.stats.magic}</span>
                  <span className={styles.statLabel}>Magic</span>
                </div>
                <div className={styles.stat}>
                  <Heart size={16} className={styles.statIcon} />
                  <span>{player.stats.resistance}</span>
                  <span className={styles.statLabel}>Resistance</span>
                </div>
                <div className={styles.stat}>
                  <Zap size={16} className={styles.statIcon} />
                  <span>{player.stats.agility}</span>
                  <span className={styles.statLabel}>Agility</span>
                </div>
              </div>
            </div>
          )}
          
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

      <main className={styles.mainContent}>{children}</main>
    </div>
  );
};
