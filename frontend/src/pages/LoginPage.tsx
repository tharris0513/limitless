import React, { useState, useEffect } from 'react';
import GameAPI from '../services/api';
import styles from './LoginPage.module.css';

interface LoginPageProps {
  onLogin: (token: string) => void;
}

export const LoginPage: React.FC<LoginPageProps> = ({ onLogin }) => {
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState('');
  const [showFallback, setShowFallback] = useState(false);

  useEffect(() => {
    const handleDiscordCallback = async (code: string, state?: string) => {
      try {
        setLoading(true);
        setError('');
        
        const response = await GameAPI.handleDiscordCallback(code, state);
        
        // Clear URL parameters
        window.history.replaceState({}, document.title, window.location.pathname);
        
        localStorage.setItem('authToken', response.token);
        onLogin(response.token);
      } catch (err: unknown) {
        console.error('Discord callback error:', err);
        setError('Discord authentication failed');
        setLoading(false);
      }
    };

    // Check if this is a callback from Discord OAuth
    const urlParams = new URLSearchParams(window.location.search);
    const code = urlParams.get('code');
    const state = urlParams.get('state');
    
    if (code) {
      // Schedule the callback to run after render to avoid cascading renders
      setTimeout(() => {
        handleDiscordCallback(code, state || undefined);
      }, 0);
    }
  }, [onLogin]);

  const handleDiscordLogin = async () => {
    try {
      setLoading(true);
      setError('');
      
      const { auth_url } = await GameAPI.getDiscordAuthUrl();
      
      // Redirect to Discord OAuth
      window.location.href = auth_url;
    } catch (err: unknown) {
      console.error('Discord login error:', err);
      setError('Failed to start Discord authentication');
      setLoading(false);
    }
  };


  const handleDemoLogin = async () => {
    try {
      setLoading(true);
      setError('');
      
      const response = await GameAPI.login('demo', 'demo');
      localStorage.setItem('authToken', response.token);
      onLogin(response.token);
    } catch (err: unknown) {
      console.error('Demo login error:', err);
      setError('Demo login failed');
      setLoading(false);
    }
  };

  return (
    <div className={styles.loginContainer}>
      <h1 className={styles.title}>🏰 Limitless</h1>
      
      <div className={styles.welcomeText}>
        Welcome to the <span className={styles.highlight}>Limitless</span> realm!<br />
        A text-based adventure inspired by Kingdom of Loathing.<br />
        <br />
        Connect your Discord account to begin your journey.
      </div>

      {loading && <div className={styles.loadingMessage}>Authenticating...</div>}

      {!loading && (
        <>
          <button onClick={handleDiscordLogin} disabled={loading} className={styles.discordButton}>
            <div className={styles.discordIcon} />
            Login with Discord
          </button>

          <div className={styles.orDivider}>or</div>

          {!showFallback ? (
            <button onClick={() => setShowFallback(true)} className={styles.fallbackButton}>
              Use Demo Account
            </button>
          ) : (
            <button onClick={handleDemoLogin} disabled={loading} className={styles.fallbackButton}>
              Login as Demo User
            </button>
          )}
        </>
      )}

      {error && <div className={styles.errorMessage}>{error}</div>}
    </div>
  );
};
