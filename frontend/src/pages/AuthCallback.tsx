import { useEffect, useState } from 'react';
import { useLocation, useNavigate } from 'react-router-dom';
import GameAPI from '../services/api';
import styles from './AuthCallback.module.css';

const AuthCallback = () => {
  const location = useLocation();
  const navigate = useNavigate();
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    const processAuth = async () => {
      const urlParams = new URLSearchParams(location.search);
      const token = urlParams.get('token');

      if (token) {
        try {
          // Store the token
          localStorage.setItem('authToken', token);
          
          // Verify the token works by trying to load user data
          await GameAPI.getUser();
          
          // Token is valid, redirect to main app
          navigate('/', { replace: true });
        } catch (error: any) {
          console.error('Failed to validate auth token:', error);
          setError('Authentication failed. Please try logging in again.');
          // Token is invalid or expired, remove it and redirect to login after showing error
          localStorage.removeItem('authToken');
          setTimeout(() => {
            navigate('/login', { replace: true });
          }, 2000);
        }
      } else {
        // No token, redirect to login
        console.error('No auth token received from Discord callback');
        setError('No authentication token received.');
        setTimeout(() => {
          navigate('/login', { replace: true });
        }, 2000);
      }
    };

    processAuth();
  }, [location.search, navigate]);

  return (
    <div className={styles.container}>
      <div className={styles.message}>
        {error ? (
          <>
            <p style={{ color: '#ff6b6b' }}>❌ {error}</p>
            <p>Redirecting to login...</p>
          </>
        ) : (
          <>
            <p>Processing Discord authentication...</p>
            <div className={styles.loading}>Loading...</div>
          </>
        )}
      </div>
    </div>
  );
};

export default AuthCallback;