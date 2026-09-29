import { describe, expect, it, beforeEach } from 'vitest';
import {
  clearStoredToken,
  getStoredToken,
  setStoredToken,
  formatAuthHeader,
} from './storage';

describe('auth storage', () => {
  beforeEach(() => {
    clearStoredToken();
  });

  it('stores access token in memory and localStorage', () => {
    setStoredToken('abc');
    expect(getStoredToken()).toBe('abc');
    expect(localStorage.getItem('rimskiy_access_token')).toBe('abc');
    clearStoredToken();
    expect(getStoredToken()).toBeNull();
    expect(localStorage.getItem('rimskiy_access_token')).toBeNull();
  });

  it('formats bearer header', () => {
    expect(formatAuthHeader('token')).toBe('Bearer token');
    expect(formatAuthHeader('Bearer x')).toBe('Bearer x');
  });
});
