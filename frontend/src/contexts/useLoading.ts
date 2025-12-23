import { useContext } from 'react';
import {
  LoadingContext,
  type LoadingContextType,
} from './LoadingContextDefinition';

// Custom hook to use the loading context
export const useLoading = (): LoadingContextType => {
  const context = useContext(LoadingContext);
  if (!context) {
    throw new Error('useLoading must be used within a LoadingProvider');
  }
  return context;
};

// Export loading key constants if they exist
export const LoadingKeys = {
  // Authentication
  AUTH_CHECK: 'auth.check',
  AUTH_LOGIN: 'auth.login',
  AUTH_LOGOUT: 'auth.logout',

  // User
  USER_FETCH: 'user.fetch',

  // Characters
  CHARACTERS_FETCH: 'characters.fetch',
  CHARACTERS_CREATE: 'characters.create',
  CHARACTERS_UPDATE: 'characters.update',

  // Adventures
  ADVENTURE_START: 'adventure.start',
  ADVENTURE_COMPLETE: 'adventure.complete',

  // Shop
  SHOP_PURCHASE: 'shop.purchase',

  // Items
  ITEM_USE: 'item.use',
  ITEM_EQUIP: 'item.equip',
} as const;
