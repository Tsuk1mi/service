import { describe, expect, it } from 'vitest';
import { authChannelHint, parseIntegrations } from './integrations';

describe('parseIntegrations', () => {
  it('defaults when integrations missing', () => {
    expect(parseIntegrations({} as never)).toEqual({
      sms: false,
      telegram: false,
      ocr: false,
      fcm: false,
      redis: false,
      rabbitmq: false,
    });
  });

  it('maps flags from server-info', () => {
    expect(
      parseIntegrations({
        server_url: 'http://localhost',
        port: 8080,
        server_version: '0.1.0',
        integrations: {
          sms: false,
          telegram: true,
          ocr: true,
          fcm: false,
          redis: true,
          rabbitmq: true,
        },
      }),
    ).toMatchObject({ telegram: true, ocr: true, sms: false });
  });
});

describe('authChannelHint', () => {
  it('prefers telegram-only when SMS off', () => {
    expect(
      authChannelHint({
        sms: false,
        telegram: true,
        ocr: false,
        fcm: false,
        redis: false,
        rabbitmq: false,
      }),
    ).toContain('Telegram');
  });
});
