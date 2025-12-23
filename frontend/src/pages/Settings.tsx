import React from 'react';
import { ArrowLeft } from 'lucide-react';
import styles from './Settings.module.css';

interface SettingsProps {
  onBack: () => void;
}

export const Settings: React.FC<SettingsProps> = ({ onBack }) => {
  return (
    <div className={styles.settingsContainer}>
      <div className={styles.header}>
        <button className={styles.backButton} onClick={onBack}>
          <ArrowLeft size={20} />
          <span>Back</span>
        </button>
        <h1 className={styles.title}>⚙️ Settings</h1>
      </div>
      
      <div className={styles.settingsContent}>
        <p className={styles.emptyMessage}>Settings coming soon...</p>
      </div>
    </div>
  );
};
