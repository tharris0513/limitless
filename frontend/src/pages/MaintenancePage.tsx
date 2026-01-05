import React from 'react';
import styles from './MaintenancePage.module.css';

export const MaintenancePage: React.FC = () => {
  return (
    <div className={styles.container}>
      <div className={styles.content}>
        <h1 className={styles.title}>🛠️ SYSTEM MAINTENANCE</h1>
        <div className={styles.message}>
          <p className={styles.mainText}>
            Limitless is currently undergoing scheduled maintenance.
          </p>
          <p className={styles.subText}>
            We're working hard to improve your gaming experience and will be back online as soon as possible.
          </p>
        </div>
        <div className={styles.statusBox}>
          <div className={styles.statusIndicator}>
            <span className={styles.pulse}></span>
            <span className={styles.statusText}>System Status: Maintenance Mode</span>
          </div>
        </div>
        <p className={styles.footer}>
          Thank you for your patience. Please check back shortly.
        </p>
      </div>
    </div>
  );
};
