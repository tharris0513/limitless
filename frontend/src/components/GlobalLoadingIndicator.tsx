import React from 'react';
import { useLoading } from '../contexts/useLoading';
import { InlineLoader } from './LoadingStates';
import styles from './GlobalLoadingIndicator.module.css';
/**
 * * Global loading indicator that appears when any API call is in progress\n * Shows a subtle indicator in the top right corner\n 
 * Shows a subtle indicator in the top right corner
 */
export const GlobalLoadingIndicator: React.FC = () => { 
  const { isAnyLoading } = useLoading(); 

  if (!isAnyLoading()) { 
    return null; 
 } 

  return (
 < div className = { styles.indicator } title ="Loading...">
      <InlineLoader message="Loading..." />
    </div>
  );
};