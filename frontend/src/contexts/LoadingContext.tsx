import React, { useReducer, type ReactNode } from 'react';
import { LoadingContext, type LoadingState } from './LoadingContextDefinition';

interface LoadingAction {
  type: 'START_LOADING' | 'STOP_LOADING';
  key: string;
}

// Reducer to manage loading states
const loadingReducer = (state: LoadingState, action: LoadingAction): LoadingState => {
  switch (action.type) {
    case 'START_LOADING':
      return { ...state, [action.key]: true };
    case 'STOP_LOADING':
      return { ...state, [action.key]: false };
    default:
      return state;
  }
};

// Provider component
export const LoadingProvider: React.FC<{ children: ReactNode }> = ({ children }) => {
  const [loading, dispatch] = useReducer(loadingReducer, {});

  const startLoading = (key: string) => {
    dispatch({ type: 'START_LOADING', key });
  };

  const stopLoading = (key: string) => {
    dispatch({ type: 'STOP_LOADING', key });
  };

  const isLoading = (key: string) => {
    return !!loading[key];
  };

  const isAnyLoading = () => {
    return Object.values(loading).some(Boolean);
  };

  const value = {
    loading,
    startLoading,
    stopLoading,
    isLoading,
    isAnyLoading,
  };

  return (
    <LoadingContext.Provider value={value}>
      {children}
    </LoadingContext.Provider>
  );
};