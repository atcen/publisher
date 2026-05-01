import { test, expect } from '@playwright/test';

test.describe('Smoke Test', () => {
  test('Page loads and responds', async ({ page }) => {
    const response = await page.goto('/', { waitUntil: 'domcontentloaded' });
    expect(response?.status()).toBeLessThan(400);
    await expect(page).toHaveTitle(/Publisher|web/i);
  });

  test('App container exists in DOM', async ({ page }) => {
    await page.goto('/', { waitUntil: 'domcontentloaded' });
    const appContainer = page.locator('#app');
    await expect(appContainer).toHaveCount(1);
  });

  test('Page HTML structure is valid', async ({ page }) => {
    await page.goto('/', { waitUntil: 'domcontentloaded' });
    const body = page.locator('body');
    await expect(body).toHaveCount(1);
    const viewport = page.locator('meta[name="viewport"]');
    await expect(viewport).toHaveCount(1);
  });
});
