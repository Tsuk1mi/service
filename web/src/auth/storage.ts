let accessToken: string | null = null;

export function getStoredToken(): string | null {
  return accessToken;
}

export function getStoredRefreshToken(): string | null {
  return null;
}

export function setStoredToken(token: string): void {
  accessToken = token;
}

export function setStoredRefreshToken(_token: string): void {
  // refresh token хранится в httpOnly cookie на сервере
}

export function setStoredTokens(access: string, _refresh: string): void {
  accessToken = access;
}

export function clearStoredToken(): void {
  accessToken = null;
}

export function formatAuthHeader(token: string): string {
  return token.startsWith('Bearer ') ? token : `Bearer ${token}`;
}

export function getBearerToken(): string | null {
  const token = getStoredToken();
  return token ? formatAuthHeader(token) : null;
}
