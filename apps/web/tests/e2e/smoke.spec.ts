import { test, expect } from '@playwright/test';

test.describe('Smoke Test', () => {
  test.beforeEach(async ({ page }) => {
    // Mock Tauri invoke to prevent errors in browser environment
    await page.addInitScript(() => {
      (window as any).__TAURI_INTERNALS__ = {};
      (window as any).__TAURI__ = {
        invoke: async (cmd: string, args?: any) => {
          console.log(`Mocked invoke: ${cmd}`, args);
          if (cmd === 'load_preferences') {
            return { theme: 'dark', default_unit: 'pt', autosave_interval: 60, recent_files: [] };
          }
          if (cmd === 'check_recovery_file') {
            return null;
          }
          return {};
        }
      };
      // For @tauri-apps/api/core
      (window as any).__TAURI_INTERNALS__.invoke = (window as any).__TAURI__.invoke;
    });
  });

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
