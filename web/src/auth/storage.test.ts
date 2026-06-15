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

  it('stores access token in memory', () => {
    setStoredToken('abc');
    expect(getStoredToken()).toBe('abc');
    clearStoredToken();
    expect(getStoredToken()).toBeNull();
  });

  it('formats bearer header', () => {
    expect(formatAuthHeader('token')).toBe('Bearer token');
    expect(formatAuthHeader('Bearer x')).toBe('Bearer x');
  });
});
