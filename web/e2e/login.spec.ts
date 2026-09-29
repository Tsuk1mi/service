import { test, expect } from '@playwright/test';

test('login form is visible', async ({ page }) => {
  await page.goto('/login');
  await expect(page).toHaveURL(/login/);
  await expect(page.getByRole('heading', { name: 'Rimskiy' })).toBeVisible();
  await expect(page.getByLabel('Телефон')).toBeVisible();
  await expect(page.getByRole('button', { name: 'Получить код' })).toBeVisible();
});

test('protected route redirects to login', async ({ page }) => {
  await page.goto('/profile');
  await expect(page).toHaveURL(/login/);
  await expect(page.getByLabel('Телефон')).toBeVisible();
});

test('home smoke with mock localStorage JWT', async ({ page }) => {
  await page.route('**/api/users/me', async (route) => {
    await route.fulfill({
      status: 200,
      contentType: 'application/json',
      body: JSON.stringify({
        id: '00000000-0000-0000-0000-000000000001',
        plate: 'A123BC77',
        show_contacts: true,
        created_at: new Date().toISOString(),
      }),
    });
  });

  await page.route('**/server-info', async (route) => {
    await route.fulfill({
      status: 200,
      contentType: 'application/json',
      body: JSON.stringify({
        server_url: 'http://localhost',
        port: 8080,
        server_version: '0.1.0-test',
        web_app_url: 'http://localhost',
        telegram_bot_username: 'rimskiy_bot',
        integrations: {
          sms: false,
          telegram: true,
          ocr: false,
          fcm: false,
          redis: true,
          rabbitmq: true,
        },
      }),
    });
  });

  await page.route('**/api/notifications**', async (route) => {
    await route.fulfill({
      status: 200,
      contentType: 'application/json',
      body: JSON.stringify([]),
    });
  });

  await page.addInitScript(() => {
    localStorage.setItem('rimskiy_access_token', 'e2e-mock-jwt');
  });

  await page.goto('/');
  await expect(page).toHaveURL(/\/$/);
  await expect(page.getByText('Добро пожаловать!')).toBeVisible();
  await expect(page.getByText('0.1.0-test')).toBeVisible();
  await expect(page.getByText('Быстрые действия')).toBeVisible();
});
