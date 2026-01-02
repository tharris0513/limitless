import React, { useState } from 'react';
import { ArrowLeft, Users, Database, Shield, Activity, Swords, UserCog, MapPin, Ghost, Compass } from 'lucide-react';
import { ManageUsers } from './ManageUsers';
import { ManageClasses } from './ManageClasses';
import { ManageCharacters } from './ManageCharacters';
import ManageCreatures from './ManageCreatures';
import ManageAdventures from './ManageAdventures';
import ManageLocations from './ManageLocations';
import styles from './AdminPanel.module.css';

interface AdminPanelProps {
  onBack: () => void;
  onLogout?: () => void;
  onViewChange?: (view: AdminView) => void; // Notify parent when view changes
  requestedView?: AdminView; // Allow parent to request a specific view
}

type AdminView = 'menu' | 'users' | 'classes' | 'characters' | 'creatures' | 'adventures' | 'locations' | 'database' | 'permissions' | 'system';

export const AdminPanel: React.FC<AdminPanelProps> = ({ onBack, onLogout, onViewChange, requestedView }) => {
  const [currentView, setCurrentView] = useState<AdminView>('menu');

  // Respond to parent's requested view changes
  React.useEffect(() => {
    if (requestedView && requestedView !== currentView) {
      setCurrentView(requestedView);
    }
  }, [requestedView, currentView]);

  // Notify parent when view changes
  const handleViewChange = (view: AdminView) => {
    setCurrentView(view);
    if (onViewChange) {
      onViewChange(view);
    }
  };

  // Render the appropriate view
  if (currentView === 'users') {
    return <ManageUsers onBack={() => handleViewChange('menu')} onLogout={onLogout} />;
  }

  if (currentView === 'classes') {
    return <ManageClasses onBack={() => handleViewChange('menu')} onLogout={onLogout} />;
  }

  if (currentView === 'characters') {
    return <ManageCharacters onBack={() => handleViewChange('menu')} onLogout={onLogout} />;
  }

  if (currentView === 'creatures') {
    return <ManageCreatures onBack={() => handleViewChange('menu')} onLogout={onLogout} />;
  }

  if (currentView === 'adventures') {
    return <ManageAdventures onBack={() => handleViewChange('menu')} onLogout={onLogout} />;
  }

  if (currentView === 'locations') {
    return <ManageLocations onBack={() => handleViewChange('menu')} onLogout={onLogout} />;
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
        <div className={styles.section}>
          <h2 className={styles.sectionTitle}>📋 Manage Resources</h2>
          <div className={styles.adminGrid}>
            <button className={styles.adminActionButton} onClick={() => handleViewChange('users')}>
              <Users size={32} />
              <span className={styles.buttonLabel}>Manage Users</span>
              <span className={styles.buttonDescription}>View and edit user accounts</span>
            </button>

            <button className={styles.adminActionButton} onClick={() => handleViewChange('characters')}>
              <UserCog size={32} />
              <span className={styles.buttonLabel}>Manage Characters</span>
              <span className={styles.buttonDescription}>View and manage all characters</span>
            </button>

            <button className={styles.adminActionButton}>
              <Database size={32} />
              <span className={styles.buttonLabel}>Database Tools</span>
              <span className={styles.buttonDescription}>Query and manage database</span>
            </button>
            <button className={styles.adminActionButton} onClick={() => handleViewChange('creatures')}>
              <Ghost size={32} />
              <span className={styles.buttonLabel}>Manage Creatures</span>
              <span className={styles.buttonDescription}>Create enemies and NPCs</span>
            </button>
            <button className={styles.adminActionButton} onClick={() => handleViewChange('adventures')}>
              <Compass size={32} />
              <span className={styles.buttonLabel}>Manage Adventures</span>
              <span className={styles.buttonDescription}>Create noncombat encounters</span>
            </button>

            <button className={styles.adminActionButton} onClick={() => handleViewChange('locations')}>
              <MapPin size={32} />
              <span className={styles.buttonLabel}>Manage Locations</span>
              <span className={styles.buttonDescription}>Create areas and assign content</span>
            </button>
          </div>
        </div>

        <div className={styles.section}>
          <h2 className={styles.sectionTitle}>⚙️ Create Resources</h2>
          <div className={styles.adminGrid}>
            <button className={styles.adminActionButton} onClick={() => handleViewChange('classes')}>
              <Swords size={32} />
              <span className={styles.buttonLabel}>Manage Classes</span>
              <span className={styles.buttonDescription}>Create and edit character classes</span>
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
