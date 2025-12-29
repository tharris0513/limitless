import React, { useState, useEffect, useRef } from 'react';
import { LoadingButton } from '../components/LoadingStates';
import type { User } from '../types/game';
import GameAPI from '../services/api';
import styles from './AccountSetup.module.css';

interface AccountSetupProps {
  user: User;
  onSetupComplete: (userData: { username: string; dateOfBirth: string }) => void;
}

export const AccountSetup: React.FC<AccountSetupProps> = ({
  user,
  onSetupComplete,
}) => {
  const [formData, setFormData] = useState({
    username: '',
    dateOfBirth: '',
    agreedToTerms: false,
  });
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState('');
  const [checkingUsername, setCheckingUsername] = useState(false);
  const [usernameAvailable, setUsernameAvailable] = useState<boolean | null>(null);
  const usernameCheckTimeout = useRef<number | undefined>(undefined);

  // Debounced username availability check
  useEffect(() => {
    // Clear previous timeout
    if (usernameCheckTimeout.current) {
      clearTimeout(usernameCheckTimeout.current);
    }

    // Reset state if username is empty or invalid
    if (!formData.username || formData.username.length < 3) {
      setUsernameAvailable(null);
      return;
    }

    // Set timeout to check username after user stops typing
    usernameCheckTimeout.current = setTimeout(async () => {
      try {
        setCheckingUsername(true);
        const available = await GameAPI.checkUsernameAvailable(formData.username);
        setUsernameAvailable(available);
      } catch (err) {
        console.error('Failed to check username availability:', err);
        // Don't block the user if check fails
        setUsernameAvailable(null);
      } finally {
        setCheckingUsername(false);
      }
    }, 500); // Wait 500ms after user stops typing

    return () => {
      if (usernameCheckTimeout.current) {
        clearTimeout(usernameCheckTimeout.current);
      }
    };
  }, [formData.username]);

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault();

    // Validation
    if (!formData.username.trim()) {
      setError('Please enter a username');
      return;
    }

    if (formData.username.length < 3) {
      setError('Username must be at least 3 characters long');
      return;
    }

    if (formData.username.length > 20) {
      setError('Username must be no more than 20 characters');
      return;
    }

    if (!/^[a-zA-Z0-9_-]+$/.test(formData.username)) {
      setError('Username can only contain letters, numbers, underscores, and hyphens');
      return;
    }

    if (usernameAvailable === false) {
      setError('This username is already taken. Please choose another.');
      return;
    }

    if (!formData.dateOfBirth) {
      setError('Please enter your date of birth');
      return;
    }

    // Age validation - must be at least 13 years old
    const birthDate = new Date(formData.dateOfBirth);
    const today = new Date();
    const age = today.getFullYear() - birthDate.getFullYear();
    const monthDiff = today.getMonth() - birthDate.getMonth();
    const actualAge = monthDiff < 0 || (monthDiff === 0 && today.getDate() < birthDate.getDate()) 
      ? age - 1 
      : age;

    if (actualAge < 13) {
      setError('You must be at least 13 years old to use this service');
      return;
    }

    if (!formData.agreedToTerms) {
      setError('You must agree to the Terms of Service and Privacy Policy');
      return;
    }

    try {
      setLoading(true);
      setError('');

      // Call the onSetupComplete callback with the form data
      onSetupComplete({
        username: formData.username,
        dateOfBirth: formData.dateOfBirth,
      });
    } catch (err: unknown) {
      console.error('Account setup error:', err);
      let errorMessage = 'Failed to complete account setup. Please try again.';
      
      if (err && typeof err === 'object') {
        if ('response' in err && err.response && typeof err.response === 'object') {
          if ('data' in err.response && err.response.data && typeof err.response.data === 'object') {
            if ('error' in err.response.data && typeof err.response.data.error === 'string') {
              errorMessage = err.response.data.error;
            }
          }
        } else if ('message' in err && typeof err.message === 'string') {
          errorMessage = err.message;
        }
      }
      
      setError(errorMessage);
    } finally {
      setLoading(false);
    }
  };

  const handleChange = (
    e: React.ChangeEvent<HTMLInputElement>
  ) => {
    const { name, value, type, checked } = e.target;
    setFormData(prev => ({
      ...prev,
      [name]: type === 'checkbox' ? checked : value,
    }));
    // Clear error when user starts typing
    if (error) setError('');
  };

  return (
    <div className={styles.container}>
      <div className={styles.card}>
        <h1 className={styles.title}>⚔️ Complete Your Account</h1>

        <div className={styles.welcomeText}>
          Welcome, <span className={styles.highlight}>{user.discordName}</span>!
          <br />
          Before you begin your adventure, we need a few more details.
        </div>

        <form onSubmit={handleSubmit} className={styles.form}>
          <div className={styles.formGroup}>
            <label htmlFor="username" className={styles.label}>
              Username *
            </label>
            <input
              type="text"
              id="username"
              name="username"
              value={formData.username}
              onChange={handleChange}
              placeholder="Choose your username"
              className={styles.input}
              maxLength={20}
              required
              disabled={loading}
              autoFocus
            />
            {checkingUsername && (
              <p className={styles.checking}>Checking availability...</p>
            )}
            {!checkingUsername && usernameAvailable === true && formData.username.length >= 3 && (
              <p className={styles.available}>✓ Username is available!</p>
            )}
            {!checkingUsername && usernameAvailable === false && (
              <p className={styles.unavailable}>✗ Username is already taken</p>
            )}
            <p className={styles.hint}>
              3-20 characters, letters, numbers, underscores, and hyphens only
            </p>
          </div>

          <div className={styles.formGroup}>
            <label htmlFor="dateOfBirth" className={styles.label}>
              Date of Birth *
            </label>
            <input
              type="date"
              id="dateOfBirth"
              name="dateOfBirth"
              value={formData.dateOfBirth}
              onChange={handleChange}
              className={styles.input}
              max={new Date().toISOString().split('T')[0]}
              required
              disabled={loading}
            />
            <p className={styles.hint}>
              You must be at least 13 years old to play
            </p>
          </div>

          <div className={styles.termsGroup}>
            <label className={styles.checkboxLabel}>
              <input
                type="checkbox"
                name="agreedToTerms"
                checked={formData.agreedToTerms}
                onChange={handleChange}
                className={styles.checkbox}
                required
                disabled={loading}
              />
              <span className={styles.checkboxText}>
                I agree to the{' '}
                <a href="/terms" target="_blank" className={styles.link}>
                  Terms of Service
                </a>{' '}
                and{' '}
                <a href="/privacy" target="_blank" className={styles.link}>
                  Privacy Policy
                </a>
              </span>
            </label>
          </div>

          {error && <div className={styles.error}>{error}</div>}

          <LoadingButton
            type="submit"
            loading={loading}
            className={styles.submitButton}
            disabled={loading}
          >
            Continue to Game
          </LoadingButton>
        </form>

        <div className={styles.footer}>
          <p className={styles.footerText}>
            Your information is secure and will never be shared with third parties.
          </p>
        </div>
      </div>
    </div>
  );
};
