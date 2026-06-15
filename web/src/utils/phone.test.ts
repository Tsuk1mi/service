import { describe, expect, it } from 'vitest';
import { normalizePhone, validatePhone } from './phone';

describe('phone utils', () => {
  it('normalizes 8-prefix to +7', () => {
    expect(normalizePhone('89001234567')).toBe('+79001234567');
  });

  it('validates russian phone', () => {
    expect(validatePhone('+79001234567')).toBe(true);
    expect(validatePhone('123')).toBe(false);
  });
});
