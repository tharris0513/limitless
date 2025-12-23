import { createContext } from 'react';

// Types for loading states
export interface LoadingState {
  [key: string]: boolean;
}

export interface LoadingContextType {
  loading: LoadingState;
  startLoading: (key: string) => void;
  stopLoading: (key: string) => void;
  isLoading: (key: string) => boolean;
  isAnyLoading: () => boolean;
}

// Create the context
export const LoadingContext = createContext<LoadingContextType | undefined>(undefined);