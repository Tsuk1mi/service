const ACCESS_TOKEN_KEY = 'rimskiy_access_token';

let accessToken: string | null = null;

function readPersistedToken(): string | null {
  try {
    return localStorage.getItem(ACCESS_TOKEN_KEY);
  } catch {
    return null;
  }
}

function persistToken(token: string | null): void {
  try {
    if (token) {
      localStorage.setItem(ACCESS_TOKEN_KEY, token);
    } else {
      localStorage.removeItem(ACCESS_TOKEN_KEY);
    }
  } catch {
    // ignore quota / private mode
  }
}

export function getStoredToken(): string | null {
  if (accessToken) return accessToken;
  accessToken = readPersistedToken();
  return accessToken;
}

export function getStoredRefreshToken(): string | null {
  return null;
}

export function setStoredToken(token: string): void {
  accessToken = token;
  persistToken(token);
}

export function setStoredRefreshToken(_token: string): void {
  void _token;
  // refresh token хранится в httpOnly cookie на сервере
}

export function setStoredTokens(access: string, _refresh: string): void {
  void _refresh;
  accessToken = access;
  persistToken(access);
}

export function clearStoredToken(): void {
  accessToken = null;
  persistToken(null);
}

export function formatAuthHeader(token: string): string {
  return token.startsWith('Bearer ') ? token : `Bearer ${token}`;
}

export function getBearerToken(): string | null {
  const token = getStoredToken();
  return token ? formatAuthHeader(token) : null;
}
