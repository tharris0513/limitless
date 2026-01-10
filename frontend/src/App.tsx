import axios from 'axios';
import { useEffect, useState } from 'react';
import { Navigate, Route, BrowserRouter as Router, Routes } from 'react-router-dom';
import './App.css';
import { GameLayout } from './components/GameLayout';
import { GlobalLoadingIndicator } from './components/GlobalLoadingIndicator';
import { ErrorScreen, LoadingOverlay } from './components/LoadingStates';
import { GameStateProvider } from './contexts/GameStateContext';
import { LoadingProvider } from './contexts/LoadingContext';
import { AccountSetup } from './pages/AccountSetup';
import { AdminPanel } from './pages/AdminPanel';
import AuthCallback from './pages/AuthCallback';
import { BannedUser } from './pages/BannedUser';
import { CharacterCreation } from './pages/CharacterCreation';
import { CharacterSelection } from './pages/CharacterSelection';
import { GamePage } from './pages/GamePage';
import { LoginPage } from './pages/LoginPage';
import { MaintenancePage } from './pages/MaintenancePage';
import { PrivacyPolicy } from './pages/PrivacyPolicy';
import { Settings } from './pages/Settings';
import { TermsOfService } from './pages/TermsOfService';
import GameAPI from './services/api';
import { type Character, type User } from './types/game';

function App() {
  const [user, setUser] = useState<User | null>(null);
  const [characters, setCharacters] = useState<Character[]>([]);
  const [selectedCharacter, setSelectedCharacter] = useState<Character | null>(null);
  const [isAuthenticated, setIsAuthenticated] = useState(false);
  const [needsAccountSetup, setNeedsAccountSetup] = useState(false);
  const [needsCharacterCreation, setNeedsCharacterCreation] = useState(false);
  const [loading, setLoading] = useState(true);
  const [loadingMessage, setLoadingMessage] = useState('Initializing...');
  const [charactersLoading, _setCharactersLoading] = useState(false);
  const [_authLoading, setAuthLoading] = useState(false);
  const [error, setError] = useState<{ title: string; message: string } | null>(null);
  const [showSettings, setShowSettings] = useState(false);
  const [showAdminPanel, setShowAdminPanel] = useState(false);
  const [adminView, setAdminView] = useState<'menu' | 'users' | 'classes'>('menu');
  const [maintenanceMode, setMaintenanceMode] = useState(false);

  useEffect(() => {
    // Check if user is already logged in
    checkAuthentication();

    // Periodically check maintenance mode (every 30 seconds)
    const maintenanceCheckInterval = setInterval(async () => {
      try {
        const maintenanceStatus = await GameAPI.checkMaintenanceMode();
        setMaintenanceMode(maintenanceStatus.maintenanceMode);
      } catch (error) {
        console.error('Failed to check maintenance mode:', error);
      }
    }, 30000); // 30 seconds

    return () => clearInterval(maintenanceCheckInterval);
  }, []); // Empty dependency array to run only once on mount

  const checkAuthentication = async () => {
    setAuthLoading(true);
    setLoading(true);
    setLoadingMessage('Checking authentication...');
    setError(null); // Clear any previous errors
    
    // First check maintenance mode
    try {
      const maintenanceStatus = await GameAPI.checkMaintenanceMode();
      setMaintenanceMode(maintenanceStatus.maintenanceMode);
    } catch (error) {
      console.error('Failed to check maintenance mode:', error);
      // Continue with authentication even if maintenance check fails
    }
    
    try {
      // First check localStorage for existing token
      const token = localStorage.getItem('authToken');
      if (token) {
        setLoadingMessage('Validating stored authentication...');
        await loadUserData();
        return;
      }

      // If no localStorage token, check for cookie-based auth
      setLoadingMessage('Checking for existing session...');
      const authData = await GameAPI.getAuthFromCookie();
      if (authData.token) {
        // Store token in localStorage for future use
        localStorage.setItem('authToken', authData.token);
        setLoadingMessage('Loading user profile...');
        await loadUserData();
        return;
      }
    } catch (error: unknown) {
      // Check if it's a network/backend error
      if (axios.isAxiosError(error)) {
        if (error.code === 'ERR_NETWORK' || !error.response) {
          setError({
            title: 'Backend Unavailable',
            message: 'Unable to connect to the game server. Please make sure the backend is running and try again.'
          });
          return;
        }
      }
      // No authentication found, continue to login
      console.log('No authentication found');
    } finally {
      setAuthLoading(false);
      setLoading(false);
    }
  };

  // Add effect to re-check auth state when localStorage changes
  useEffect(() => {
    const handleStorageChange = () => {
      const token = localStorage.getItem('authToken');
      if (token && !isAuthenticated) {
        loadUserData();
      } else if (!token && isAuthenticated) {
        resetAuthState();
      }
    };

    window.addEventListener('storage', handleStorageChange);
    return () => window.removeEventListener('storage', handleStorageChange);
  }, [isAuthenticated]);

  // Refresh character list when returning from admin panel
  useEffect(() => {
    if (!showAdminPanel && isAuthenticated && user) {
      // Refresh characters when closing admin panel
      const refreshCharacters = async () => {
        try {
          const userCharacters = await GameAPI.getUserCharacters();
          setCharacters(userCharacters);
          
          // If the selected character was deleted, clear selection
          if (selectedCharacter && !userCharacters.find(c => c.id === selectedCharacter.id)) {
            setSelectedCharacter(null);
          }
        } catch (error) {
          console.error('Failed to refresh characters:', error);
        }
      };
      refreshCharacters();
    }
  }, [showAdminPanel, isAuthenticated]);

  const loadUserData = async () => {
    setError(null); // Clear any previous errors
    try {
      // Load user and their characters
      const userData = await GameAPI.getUser();
      const userCharacters = await GameAPI.getUserCharacters();

      setUser(userData);
      setCharacters(userCharacters);
      setIsAuthenticated(true);
      
      // Check if user is banned
      if (userData.banned) {
        setNeedsAccountSetup(false);
        setNeedsCharacterCreation(false);
        return; // Will be redirected by route guard
      }
      
      // Check if user needs to complete account setup
      // Only require setup if username is missing or empty
      const hasValidUsername = userData.username && userData.username.trim().length > 0;
      const hasValidDOB = userData.dateOfBirth && userData.dateOfBirth.trim().length > 0;
      
      if (!hasValidUsername || !hasValidDOB) {
        setNeedsAccountSetup(true);
        return;
      }
      
      // Determine next step based on characters
      if (userCharacters.length === 0) {
        setNeedsCharacterCreation(true);
      } else {
        setNeedsCharacterCreation(false);
        // Auto-select last played character or show selection
        const lastPlayedCharacter = userCharacters.reduce((latest, char) => 
          new Date(char.lastPlayed) > new Date(latest.lastPlayed) ? char : latest
        );
        setSelectedCharacter(lastPlayedCharacter);
      }
    } catch (error) {
      console.error('Failed to load user data:', error);
      
      // Check if it's a network/backend error
      if (axios.isAxiosError(error)) {
        if (error.code === 'ERR_NETWORK' || !error.response) {
          setError({
            title: 'Backend Unavailable',
            message: 'Unable to connect to the game server. Please make sure the backend is running and try again.'
          });
          setLoading(false);
          return;
        }
        
        // Check if it's an auth error (401/403) vs other errors
        if (error.response?.status === 401 || error.response?.status === 403) {
          // Auth token is invalid, clear it
          resetAuthState();
        } else {
          // Other error (likely new user), show character creation
          setNeedsCharacterCreation(true);
          setIsAuthenticated(true);
        }
      }
    } finally {
      setLoading(false);
    }
  };

  const resetAuthState = () => {
    localStorage.removeItem('authToken');
    setIsAuthenticated(false);
    setUser(null);
    setCharacters([]);
    setSelectedCharacter(null);
    setNeedsCharacterCreation(false);
  };

  const handleLogin = async (token: string) => {
    localStorage.setItem('authToken', token);
    await loadUserData();
  };

  const handleLogout = async () => {
    try {
      // Call logout API endpoint
      await GameAPI.logout();
    } catch (error) {
      console.error('Logout API call failed:', error);
      // Continue with local cleanup even if API call fails
    } finally {
      // Clear all localStorage data
      localStorage.removeItem('authToken');
      localStorage.clear();

      // Clear all sessionStorage data
      sessionStorage.clear();

      // Reset all React state to initial values
      setIsAuthenticated(false);
      setUser(null);
      setCharacters([]);
      setSelectedCharacter(null);
      setNeedsCharacterCreation(false);
      setLoading(false);

      // Clear any potential cached data in memory
      // Reset axios default headers to remove stale auth tokens
      if (axios.defaults.headers.common) {
        delete axios.defaults.headers.common['Authorization'];
      }

      // Clear any browser caches if supported
      if ('caches' in window) {
        try {
          const cacheNames = await caches.keys();
          await Promise.all(
            cacheNames.map(cacheName => caches.delete(cacheName))
          );
        } catch (error) {
          console.warn('Failed to clear browser caches:', error);
        }
      }

      // Force complete page reload to ensure clean state
      // This clears any remaining JavaScript variables, closures, or cached data
      window.location.href = '/login';
    }
  };

  const handleCharacterSelect = (character: Character) => {
    setSelectedCharacter(character);
    setNeedsCharacterCreation(false);
    // Update last played timestamp
    GameAPI.updateCharacterLastPlayed(character.id);
  };

  const handleCreateNewCharacter = () => {
    setNeedsCharacterCreation(true);
    setSelectedCharacter(null);
  };

  const handleCharacterCreated = (newCharacter: Character) => {
    setCharacters(prev => [...prev, newCharacter]);
    setSelectedCharacter(newCharacter);
    setNeedsCharacterCreation(false);
  };

  const handleCharacterUpdate = (updatedCharacter: Character) => {
    setCharacters(prev => 
      prev.map(char => char.id === updatedCharacter.id ? updatedCharacter : char)
    );
    setSelectedCharacter(updatedCharacter);
  };

  const handleAccountSetupComplete = async (setupData: {
    username: string;
    dateOfBirth: string;
  }) => {
    try {
      setLoading(true);
      setLoadingMessage('Completing account setup...');
      
      // Update user with account setup data
      const updatedUser = await GameAPI.updateUser(setupData);
      
      setUser(updatedUser);
      setNeedsAccountSetup(false);
      
      // After account setup, check if they need to create a character
      if (characters.length === 0) {
        setNeedsCharacterCreation(true);
      }
    } catch (error) {
      console.error('Failed to complete account setup:', error);
      setError({
        title: 'Account Setup Failed',
        message: 'Failed to save account information. Please try again.',
      });
    } finally {
      setLoading(false);
    }
  };

  if (error) {
    return (
      <ErrorScreen
        title={error.title}
        message={error.message}
        onRetry={checkAuthentication}
        showLogout={isAuthenticated}
        onLogout={isAuthenticated ? handleLogout : undefined}
      />
    );
  }

  if (loading) {
    return <LoadingOverlay message={loadingMessage} />;
  }

  // If in maintenance mode and user is not an admin, show maintenance page
  if (maintenanceMode && (!user || !user.admin)) {
    return <MaintenancePage />;
  }

  return (
    <LoadingProvider>
      <GlobalLoadingIndicator />
      <Router>
        <Routes>
          <Route path="/auth/callback" element={<AuthCallback />} />
          <Route path="/banned" element={<BannedUser />} />
          <Route path="/login" element={
            isAuthenticated ? <Navigate to="/" replace /> : 
            <GameLayout player={null} user={null} onLogout={undefined}>
              <LoginPage onLogin={handleLogin} />
            </GameLayout>
          } />
        <Route path="/" element={
          !isAuthenticated ? (
            <Navigate to="/login" replace />
          ) : user?.banned ? (
            <Navigate to="/banned" replace />
          ) : needsAccountSetup ? (
            user ? (
              <AccountSetup
                user={user}
                onSetupComplete={handleAccountSetupComplete}
              />
            ) : (
              <div>Loading user data...</div>
            )
          ) : showSettings ? (
            <GameLayout player={selectedCharacter} user={user} onLogout={handleLogout} isAdmin={user?.admin}>
              <Settings onBack={() => setShowSettings(false)} />
            </GameLayout>
          ) : showAdminPanel ? (
            <GameLayout 
              key={adminView} // Force re-render when adminView changes
              player={selectedCharacter}
              user={user}
              onLogout={handleLogout} 
              isAdmin={user?.admin}
              onAdminClick={() => {
                // Always go to admin menu (or stay if already there)
                setAdminView('menu');
              }}
              hideCharacterInfo={true}
            >
              <AdminPanel 
                onBack={() => setShowAdminPanel(false)} 
                onLogout={handleLogout}
                onViewChange={(view) => setAdminView(view as 'menu' | 'users' | 'classes')}
                requestedView={adminView}
              />
            </GameLayout>
          ) : needsCharacterCreation || characters.length === 0 ? (
            user ? (
              <GameLayout 
                player={selectedCharacter}
                user={user}
                onLogout={handleLogout} 
                onSettingsClick={() => setShowSettings(true)} 
                isAdmin={user.admin}
                onAdminClick={() => setShowAdminPanel(true)}
              >
                <CharacterCreation 
                  user={user}
                  onCharacterCreated={handleCharacterCreated} 
                />
              </GameLayout>
            ) : (
              <div>Loading user data...</div>
            )
          ) : !selectedCharacter ? (
            user ? (
              <GameLayout 
                player={null}
                user={user}
                onLogout={handleLogout} 
                onSettingsClick={() => setShowSettings(true)} 
                isAdmin={user.admin}
                onAdminClick={() => setShowAdminPanel(true)}
              >
                <CharacterSelection
                  user={user}
                  characters={characters}
                  isLoading={charactersLoading}
                  onCharacterSelect={handleCharacterSelect}
                  onCreateNewCharacter={handleCreateNewCharacter}
                />
              </GameLayout>
            ) : (
              <div>Loading user data...</div>
            )
          ) : (
            <GameLayout 
              player={selectedCharacter}
              user={user}
              onLogout={handleLogout} 
              onSettingsClick={() => setShowSettings(true)} 
              isAdmin={user?.admin}
              onAdminClick={() => setShowAdminPanel(true)}
            >
              <GameStateProvider character={selectedCharacter}>
                <GamePage
                  character={selectedCharacter}
                  onCharacterUpdate={handleCharacterUpdate}
                  onCharacterDeleted={() => setSelectedCharacter(null)}
                />
              </GameStateProvider>
            </GameLayout>
          )
        } />
        <Route path="/characters" element={
          !isAuthenticated ? (
            <Navigate to="/login" replace />
          ) : user?.banned ? (
            <Navigate to="/banned" replace />
          ) : user ? (
            <GameLayout 
              player={null}
              user={user}
              onLogout={handleLogout} 
              onSettingsClick={() => setShowSettings(true)} 
              isAdmin={user.admin}
              onAdminClick={() => setShowAdminPanel(true)}
            >
              <CharacterSelection
                user={user}
                characters={characters}
                isLoading={charactersLoading}
                onCharacterSelect={handleCharacterSelect}
                onCreateNewCharacter={handleCreateNewCharacter}
              />
            </GameLayout>
          ) : (
            <div>Loading user data...</div>
          )
        } />
        <Route path="/create-character" element={
          !isAuthenticated ? (
            <Navigate to="/login" replace />
          ) : user?.banned ? (
            <Navigate to="/banned" replace />
          ) : user ? (
            <GameLayout 
              player={selectedCharacter}
              user={user}
              onLogout={handleLogout} 
              onSettingsClick={() => setShowSettings(true)} 
              isAdmin={user.admin}
              onAdminClick={() => setShowAdminPanel(true)}
            >
              <CharacterCreation 
                user={user}
                onCharacterCreated={handleCharacterCreated} 
              />
            </GameLayout>
          ) : (
            <div>Loading user data...</div>
          )
        } />
        <Route path="/combat" element={<Navigate to="/" replace />} />
        <Route path="/adventure" element={<Navigate to="/" replace />} />
        <Route path="/terms" element={<TermsOfService />} />
        <Route path="/privacy" element={<PrivacyPolicy />} />
      </Routes>
    </Router>
  </LoadingProvider>
  );
}

export default App;
