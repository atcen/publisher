import { test, expect } from '@playwright/test';

test.describe('Smoke Test', () => {
  test('Page loads and responds', async ({ page }) => {
    const consoleErrors: string[] = [];
    const pageErrors: Error[] = [];

    // Capture console errors and exceptions
    page.on('console', (msg) => {
      if (msg.type() === 'error') {
        consoleErrors.push(msg.text());
      }
    });

    page.on('pageerror', (error) => {
      pageErrors.push(error);
    });

    const response = await page.goto('/', { waitUntil: 'domcontentloaded' });
    expect(response?.status()).toBeLessThan(400);
    await expect(page).toHaveTitle(/Publisher|web/i);

    // Verify no console errors or exceptions
    expect(consoleErrors, 'Should have no console errors').toEqual([]);
    expect(pageErrors, 'Should have no page errors').toEqual([]);
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
