import { useEffect } from 'react';
import { useLocation, useNavigate } from 'react-router-dom';
import GameAPI from '../services/api';
import styles from './AuthCallback.module.css';

const AuthCallback = () => {
  const location = useLocation();
  const navigate = useNavigate();

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
        } catch (error) {
          console.error('Failed to validate auth token:', error);
          // Token is invalid or expired, remove it and redirect to login
          localStorage.removeItem('authToken');
          navigate('/login', { replace: true });
        }
      } else {
        // No token, redirect to login
        console.error('No auth token received from Discord callback');
        navigate('/login', { replace: true });
      }
    };

    processAuth();
  }, [location.search, navigate]);

  return (
    <div className={styles.container}>
      <div className={styles.message}>
        <p>Processing Discord authentication...</p>
        <div className={styles.loading}>Loading...</div>
      </div>
    </div>
  );
};

export default AuthCallback;