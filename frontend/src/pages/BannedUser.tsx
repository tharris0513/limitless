import { useNavigate } from 'react-router-dom';
import styles from './BannedUser.module.css';

export const BannedUser = () => {
  const navigate = useNavigate();

  const handleLogout = () => {
    // Clear all authentication data
    localStorage.removeItem('authToken');
    localStorage.clear();
    sessionStorage.clear();
    
    // Redirect to login page
    navigate('/login');
    window.location.reload();
  };

  return (
    <div className={styles.bannedContainer}>
      <div className={styles.bannedBox}>
        <div className={styles.iconContainer}>
          <div className={styles.banIcon}>⛔</div>
        </div>
        
        <h1 className={styles.title}>Account Banned</h1>
        
        <div className={styles.message}>
          <p>Your account has been banned from accessing this game.</p>
          <p>This action was taken due to a violation of our terms of service.</p>
        </div>
        
        <div className={styles.infoBox}>
          <p className={styles.infoTitle}>What does this mean?</p>
          <ul className={styles.infoList}>
            <li>You cannot access the game with this account</li>
            <li>Your characters and progress are suspended</li>
            <li>This ban may be temporary or permanent</li>
          </ul>
        </div>
        
        <div className={styles.contactInfo}>
          <p>If you believe this is a mistake, please contact the game administrators.</p>
        </div>
        
        <button className={styles.logoutButton} onClick={handleLogout}>
          Return to Login
        </button>
      </div>
    </div>
  );
};
