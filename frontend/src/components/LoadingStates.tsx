import React from 'react';
import styles from './LoadingStates.module.css';

// Basic spinner component
export const Spinner: React.FC<{ size?: 'small' | 'medium' | 'large' }> = ({ 
  size = 'medium' 
}) => (
  <div className={`${styles.spinner} ${styles[size]}`}>
    <div className={styles.spinnerRing}></div>
  </div>
);

// Loading button component
export const LoadingButton: React.FC<{
  onClick?: () => void;
  loading: boolean;
  disabled?: boolean;
  children: React.ReactNode;
  className?: string;
  variant?: 'primary' | 'secondary';
  type?: 'button' | 'submit' | 'reset';
}> = ({ 
  onClick, 
  loading, 
  disabled = false, 
  children, 
  className = '',
  variant = 'primary',
  type = 'button'
}) => (
  <button
    type={type}
    onClick={onClick}
    disabled={loading || disabled}
    className={`${styles.loadingButton} ${styles[variant]} ${className}`}
  >
    {loading ? (
      <>
        <Spinner size="small" />
        <span>Loading...</span>
      </>
    ) : (
      children
    )}
  </button>
);

// Card skeleton loader
export const CardSkeleton: React.FC = () => (
  <div className={styles.cardSkeleton}>
    <div className={styles.skeletonHeader}>
      <div className={styles.skeletonAvatar}></div>
      <div className={styles.skeletonText}>
        <div className={styles.skeletonLine}></div>
        <div className={styles.skeletonLine}></div>
      </div>
    </div>
    <div className={styles.skeletonContent}>
      <div className={styles.skeletonLine}></div>
      <div className={styles.skeletonLine}></div>
      <div className={styles.skeletonLine}></div>
    </div>
  </div>
);

// Character selection skeleton
export const CharacterSkeleton: React.FC = () => (
  <div className={styles.characterSkeleton}>
    <div className={styles.skeletonAvatar}></div>
    <div className={styles.skeletonText}>
      <div className={styles.skeletonLine}></div>
      <div className={styles.skeletonLine}></div>
    </div>
    <div className={styles.skeletonStats}>
      <div className={styles.skeletonStat}></div>
      <div className={styles.skeletonStat}></div>
      <div className={styles.skeletonStat}></div>
    </div>
  </div>
);

// Full page loading overlay
export const LoadingOverlay: React.FC<{ message?: string }> = ({ 
  message = 'Loading...' 
}) => (
  <div className={styles.loadingOverlay}>
    <div className={styles.overlayContent}>
      <Spinner size="large" />
      <p>{message}</p>
    </div>
  </div>
);

// Inline loading state for forms
export const InlineLoader: React.FC<{ message: string }> = ({ message }) => (
  <div className={styles.inlineLoader}>
    <Spinner size="small" />
    <span>{message}</span>
  </div>
);

// Error screen for backend connectivity issues
export const ErrorScreen: React.FC<{
  title?: string;
  message: string;
  onRetry?: () => void;
  showLogout?: boolean;
  onLogout?: () => void;
}> = ({ 
  title = 'Connection Error',
  message,
  onRetry,
  showLogout = false,
  onLogout
}) => (
  <div className={styles.errorScreen}>
    <div className={styles.errorContent}>
      <div className={styles.errorIcon}>⚠️</div>
      <h2 className={styles.errorTitle}>{title}</h2>
      <p className={styles.errorMessage}>{message}</p>
      <div className={styles.errorActions}>
        {onRetry && (
          <button onClick={onRetry} className={styles.retryButton}>
            Try Again
          </button>
        )}
        {showLogout && onLogout && (
          <button onClick={onLogout} className={styles.logoutButton}>
            Logout
          </button>
        )}
      </div>
    </div>
  </div>
);