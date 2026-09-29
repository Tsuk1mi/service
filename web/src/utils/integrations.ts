import type { IntegrationsStatus, ServerInfoResponse } from '../api/types';

const DEFAULT_INTEGRATIONS: IntegrationsStatus = {
  sms: false,
  telegram: false,
  ocr: false,
  fcm: false,
  redis: false,
  rabbitmq: false,
};

/** Normalize `/server-info` integrations payload (missing keys → false). */
export function parseIntegrations(
  data: ServerInfoResponse | null | undefined,
): IntegrationsStatus {
  const raw = data?.integrations;
  if (!raw || typeof raw !== 'object') {
    return { ...DEFAULT_INTEGRATIONS };
  }
  return {
    sms: Boolean(raw.sms),
    telegram: Boolean(raw.telegram),
    ocr: Boolean(raw.ocr),
    fcm: Boolean(raw.fcm),
    redis: Boolean(raw.redis),
    rabbitmq: Boolean(raw.rabbitmq),
  };
}

export function authChannelHint(integrations: IntegrationsStatus): string {
  if (integrations.sms && integrations.telegram) {
    return 'Авторизация через SMS или Telegram-бота.';
  }
  if (integrations.telegram) {
    return 'Авторизация через Telegram-бота (SMS-провайдер не настроен).';
  }
  if (integrations.sms) {
    return 'Авторизация через SMS.';
  }
  return 'Авторизация через SMS или Telegram-бота (проверьте конфигурацию сервера).';
}
