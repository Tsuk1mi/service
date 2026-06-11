export function openPhone(phone: string): void {
  window.location.href = `tel:${phone.replace(/\s/g, '')}`;
}

export function openSms(phone: string, message?: string): void {
  const clean = phone.replace(/\s/g, '');
  const body = message ? `?body=${encodeURIComponent(message)}` : '';
  window.location.href = `sms:${clean}${body}`;
}

export function openTelegram(username: string): void {
  const clean = username.replace(/^@/, '');
  window.open(`https://t.me/${clean}`, '_blank', 'noopener,noreferrer');
}

export function openTelegramDeeplink(deeplink: string): void {
  window.open(deeplink, '_blank', 'noopener,noreferrer');
}
