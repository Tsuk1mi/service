export function normalizePhone(phone: string): string {
  let cleaned = phone.replace(/[^\d+]/g, '');

  if (cleaned.startsWith('+7')) {
    // already correct
  } else if (cleaned.startsWith('8') && cleaned.length > 1) {
    cleaned = '+7' + cleaned.substring(1);
  } else if (cleaned.startsWith('7') && cleaned.length > 1) {
    cleaned = '+' + cleaned;
  } else if (cleaned.length > 0 && cleaned[0] !== '+' && /\d/.test(cleaned[0])) {
    cleaned = '+7' + cleaned;
  }

  return cleaned;
}

export function validatePhone(phone: string): boolean {
  const normalized = normalizePhone(phone);
  return (
    normalized.length >= 10 &&
    (normalized.startsWith('+') ||
      normalized.startsWith('8') ||
      normalized.startsWith('7'))
  );
}

export function formatPhone(phone: string): string {
  const normalized = normalizePhone(phone);
  if (normalized.startsWith('+7') && normalized.length === 12) {
    return `+7 (${normalized.slice(2, 5)}) ${normalized.slice(5, 8)}-${normalized.slice(8, 10)}-${normalized.slice(10, 12)}`;
  }
  if (normalized.startsWith('8') && normalized.length === 11) {
    return `8 (${normalized.slice(1, 4)}) ${normalized.slice(4, 7)}-${normalized.slice(7, 9)}-${normalized.slice(9, 11)}`;
  }
  return normalized;
}
