import { createContext, useCallback, useContext, useEffect, useMemo, useState } from 'react';
import { api } from '../api/client';
import { clearStoredToken, getStoredToken, setStoredToken } from './storage';

interface AuthContextValue {
  isAuthenticated: boolean;
  isLoading: boolean;
  login: (token: string) => void;
  logout: () => void;
  checkAuth: () => Promise<boolean>;
}

const AuthContext = createContext<AuthContextValue | null>(null);

export function AuthProvider({ children }: { children: React.ReactNode }) {
  const [isAuthenticated, setIsAuthenticated] = useState(false);
  const [isLoading, setIsLoading] = useState(true);

  const checkAuth = useCallback(async (): Promise<boolean> => {
    const token = getStoredToken();
    if (!token) {
      setIsAuthenticated(false);
      return false;
    }
    try {
      await api.getProfile();
      setIsAuthenticated(true);
      return true;
    } catch {
      clearStoredToken();
      setIsAuthenticated(false);
      return false;
    }
  }, []);

  useEffect(() => {
    void checkAuth().finally(() => setIsLoading(false));
  }, [checkAuth]);

  const login = useCallback((token: string) => {
    setStoredToken(token);
    setIsAuthenticated(true);
  }, []);

  const logout = useCallback(() => {
    clearStoredToken();
    setIsAuthenticated(false);
  }, []);

  const value = useMemo(
    () => ({ isAuthenticated, isLoading, login, logout, checkAuth }),
    [isAuthenticated, isLoading, login, logout, checkAuth],
  );

  return <AuthContext.Provider value={value}>{children}</AuthContext.Provider>;
}

export function useAuth(): AuthContextValue {
  const ctx = useContext(AuthContext);
  if (!ctx) throw new Error('useAuth must be used within AuthProvider');
  return ctx;
}
