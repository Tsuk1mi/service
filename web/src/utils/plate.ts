export function normalizePlate(plate: string): string {
  return plate.replace(/[\s-]/g, '').toUpperCase();
}

export function validatePlate(plate: string): boolean {
  const normalized = normalizePlate(plate);
  if (normalized.length < 8 || normalized.length > 9) return false;

  const chars = normalized.split('');
  if (!/[A-Za-zА-Яа-я]/.test(chars[0])) return false;
  if (!/\d/.test(chars[1]) || !/\d/.test(chars[2]) || !/\d/.test(chars[3])) return false;
  if (!/[A-Za-zА-Яа-я]/.test(chars[4]) || !/[A-Za-zА-Яа-я]/.test(chars[5])) return false;

  const remaining = normalized.substring(6);
  return remaining.split('').every((c) => /\d/.test(c));
}

export function formatPlate(plate: string): string {
  const normalized = normalizePlate(plate);
  if (normalized.length === 9) {
    return `${normalized[0]} ${normalized.slice(1, 4)} ${normalized.slice(4, 6)} ${normalized.slice(6, 9)}`;
  }
  if (normalized.length === 8) {
    return `${normalized[0]} ${normalized.slice(1, 4)} ${normalized.slice(4, 6)} ${normalized.slice(6, 8)}`;
  }
  return normalized;
}
