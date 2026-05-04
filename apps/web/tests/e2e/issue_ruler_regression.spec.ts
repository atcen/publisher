import { test, expect } from '@playwright/test';

test.describe('Ruler and Guide Regression', () => {
  test.beforeEach(async ({ page }) => {
    await page.addInitScript(() => {
      const mockInvoke = async (cmd: string, args?: any) => {
        if (cmd === 'load_preferences') return { theme: 'dark', default_unit: 'pt', autosave_interval: 60, recent_files: [] };
        return {};
      };
      (window as any).__TAURI_INTERNALS__ = { invoke: mockInvoke };
      (window as any).__TAURI__ = { invoke: mockInvoke };
    });
    await page.goto('/');
    await page.waitForSelector('.toolbar', { state: 'visible' });
  });

  test('should pull a vertical guide from the ruler', async ({ page }) => {
    const leftRuler = page.locator('.left-ruler');
    await expect(leftRuler).toBeVisible();

    // 1. Pull a guide from the left ruler (Horizontal Guide)
    // We click on the left ruler and drag it onto the page
    const rulerBox = await leftRuler.boundingBox();
    const startX = rulerBox!.x + rulerBox!.width / 2;
    const startY = rulerBox!.y + 50;

    await page.mouse.move(startX, startY);
    await page.mouse.down();
    await page.mouse.move(startX + 200, startY + 100); // Drag onto page
    await page.mouse.up();

    // 2. Verify guide exists in DOM
    const guide = page.locator('.guide.horizontal');
    await expect(guide).toBeVisible();
    
    // 3. Move the guide
    const initialPos = await guide.boundingBox();
    await guide.hover();
    await page.mouse.down();
    await page.mouse.move(initialPos!.x, initialPos!.y + 50);
    await page.mouse.up();

    const finalPos = await guide.boundingBox();
    expect(finalPos!.y).toBeGreaterThan(initialPos!.y);
  });
});
