import axios from 'axios';
import { useEffect, useState } from 'react';
import { Navigate, Route, BrowserRouter as Router, Routes } from 'react-router-dom';
import './App.css';
import { GameLayout } from './components/GameLayout';
import { GlobalLoadingIndicator } from './components/GlobalLoadingIndicator';
import { ErrorScreen, LoadingOverlay } from './components/LoadingStates';
import { LoadingProvider } from './contexts/LoadingContext';
import { AdminPanel } from './pages/AdminPanel';
import AuthCallback from './pages/AuthCallback';
import { CharacterCreation } from './pages/CharacterCreation';
import { CharacterSelection } from './pages/CharacterSelection';
import { GamePage } from './pages/GamePage';
import { LoginPage } from './pages/LoginPage';
import { Settings } from './pages/Settings';
import GameAPI from './services/api';
import { type Character, type User } from './types/game';

function App() {
  const [user, setUser] = useState<User | null>(null);
  const [characters, setCharacters] = useState<Character[]>([]);
  const [selectedCharacter, setSelectedCharacter] = useState<Character | null>(null);
  const [isAuthenticated, setIsAuthenticated] = useState(false);
  const [needsCharacterCreation, setNeedsCharacterCreation] = useState(false);
  const [loading, setLoading] = useState(true);
  const [loadingMessage, setLoadingMessage] = useState('Initializing...');
  const [charactersLoading, _setCharactersLoading] = useState(false);
  const [_authLoading, setAuthLoading] = useState(false);
  const [error, setError] = useState<{ title: string; message: string } | null>(null);
  const [showSettings, setShowSettings] = useState(false);
  const [showAdminPanel, setShowAdminPanel] = useState(false);

  useEffect(() => {
    // Check if user is already logged in
    checkAuthentication();
  }, []); // Empty dependency array to run only once on mount

  const checkAuthentication = async () => {
    setAuthLoading(true);
    setLoading(true);
    setLoadingMessage('Checking authentication...');
    setError(null); // Clear any previous errors
    
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

  const loadUserData = async () => {
    setError(null); // Clear any previous errors
    try {
      console.log('Loading user data...');
      // Load user and their characters
      const userData = await GameAPI.getUser();
      const userCharacters = await GameAPI.getUserCharacters();

      console.log('User data loaded:', userData);
      console.log('User admin status:', userData.admin);
      console.log('Characters loaded:', userCharacters);

      setUser(userData);
      setCharacters(userCharacters);
      setIsAuthenticated(true);
      
      // Determine next step based on characters
      if (userCharacters.length === 0) {
        console.log('No characters found, setting needsCharacterCreation to true');
        setNeedsCharacterCreation(true);
      } else {
        console.log('Characters found, auto-selecting last played');
        setNeedsCharacterCreation(false);
        // Auto-select last played character or show selection
        const lastPlayedCharacter = userCharacters.reduce((latest, char) => 
          new Date(char.lastPlayed) > new Date(latest.lastPlayed) ? char : latest
        );
        setSelectedCharacter(lastPlayedCharacter);
        console.log('Selected character:', lastPlayedCharacter);
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
          console.log('Auth error, resetting auth state');
          resetAuthState();
        } else {
          // Other error (likely new user), show character creation
          console.log('Other error, showing character creation');
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
    console.log('App is in loading state');
    return <LoadingOverlay message={loadingMessage} />;
  }

  console.log('App render state:', {
    isAuthenticated,
    user,
    characters: characters.length,
    selectedCharacter,
    needsCharacterCreation,
    loading
  });

  return (
    <LoadingProvider>
      <GlobalLoadingIndicator />
      <Router>
        <Routes>
          <Route path="/auth/callback" element={<AuthCallback />} />
          <Route path="/login" element={
            isAuthenticated ? <Navigate to="/" replace /> : 
            <GameLayout player={null} onLogout={undefined}>
              <LoginPage onLogin={handleLogin} />
            </GameLayout>
          } />
        <Route path="/" element={
          !isAuthenticated ? (
            <Navigate to="/login" replace />
          ) : showSettings ? (
            <GameLayout player={selectedCharacter} onLogout={handleLogout} isAdmin={user?.admin}>
              <Settings onBack={() => setShowSettings(false)} />
            </GameLayout>
          ) : showAdminPanel ? (
            <GameLayout player={selectedCharacter} onLogout={handleLogout} isAdmin={user?.admin}>
              <AdminPanel onBack={() => setShowAdminPanel(false)} />
            </GameLayout>
          ) : needsCharacterCreation || characters.length === 0 ? (
            user ? (
              <GameLayout 
                player={selectedCharacter} 
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
              onLogout={handleLogout} 
              onSettingsClick={() => setShowSettings(true)} 
              isAdmin={user?.admin}
              onAdminClick={() => setShowAdminPanel(true)}
            >
              <GamePage
                player={selectedCharacter}
                onPlayerUpdate={handleCharacterUpdate}
              />
            </GameLayout>
          )
        } />
        <Route path="/characters" element={
          !isAuthenticated ? (
            <Navigate to="/login" replace />
          ) : user ? (
            <GameLayout 
              player={selectedCharacter} 
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
          ) : user ? (
            <GameLayout 
              player={selectedCharacter} 
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
      </Routes>
    </Router>
  </LoadingProvider>
  );
}

export default App;
