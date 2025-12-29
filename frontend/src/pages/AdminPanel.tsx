import React, { useState } from 'react';
import { ArrowLeft, Users, Database, Shield, Activity } from 'lucide-react';
import { ManageUsers } from './ManageUsers';
import styles from './AdminPanel.module.css';

interface AdminPanelProps {
  onBack: () => void;
  onLogout?: () => void;
}

type AdminView = 'menu' | 'users' | 'database' | 'permissions' | 'system';

export const AdminPanel: React.FC<AdminPanelProps> = ({ onBack, onLogout }) => {
  const [currentView, setCurrentView] = useState<AdminView>('menu');

  // Render the appropriate view
  if (currentView === 'users') {
    return <ManageUsers onBack={() => setCurrentView('menu')} onLogout={onLogout} />;
  }

  // Default menu view
  return (
    <div className={styles.adminContainer}>
      <div className={styles.header}>
        <button className={styles.backButton} onClick={onBack}>
          <ArrowLeft size={20} />
          <span>Back</span>
        </button>
        <h1 className={styles.title}>🛡️ Admin Panel</h1>
      </div>
      
      <div className={styles.adminContent}>
        <div className={styles.adminGrid}>
          <button className={styles.adminActionButton} onClick={() => setCurrentView('users')}>
            <Users size={32} />
            <span className={styles.buttonLabel}>Manage Users</span>
            <span className={styles.buttonDescription}>View and edit user accounts</span>
          </button>

          <button className={styles.adminActionButton}>
            <Database size={32} />
            <span className={styles.buttonLabel}>Database Tools</span>
            <span className={styles.buttonDescription}>Query and manage database</span>
          </button>

          <button className={styles.adminActionButton}>
            <Shield size={32} />
            <span className={styles.buttonLabel}>Permissions</span>
            <span className={styles.buttonDescription}>Manage admin roles</span>
          </button>

          <button className={styles.adminActionButton}>
            <Activity size={32} />
            <span className={styles.buttonLabel}>System Status</span>
            <span className={styles.buttonDescription}>View server metrics</span>
          </button>
        </div>

        <div className={styles.warningBox}>
          <span className={styles.warningIcon}>⚠️</span>
          <p className={styles.warningText}>
            You are accessing administrative functions. All actions are logged.
          </p>
        </div>
      </div>
    </div>
  );
};
