import { test, expect } from '@playwright/test';

test.describe('Text Frame Creation', () => {
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
      (window as any).__TAURI_INTERNALS__.invoke = (window as any).__TAURI__.invoke;
    });
  });

  test('should create a text frame and switch types', async ({ page }) => {
    await page.goto('/');

    // 1. Select the Text Tool (T)
    await page.click('button[title*="Text (T)"]');

    // 2. Click on a page to create a frame
    const pageElement = page.locator('.page').first();
    await pageElement.waitFor({ state: 'visible' });
    
    // Click at relative (100, 100) inside the page
    await pageElement.click({ position: { x: 100, y: 100 } });

    // 3. Verify frame is created and selected
    // We look for a frame within the page that has the selected class
    const frame = pageElement.locator('.frame.selected');
    await expect(frame).toBeVisible({ timeout: 10000 });

    // 4. Check Sidebar properties
    // The sidebar might take a moment to react to the selection
    const typeSelect = page.locator('.properties select').nth(1); 
    await expect(typeSelect).toBeVisible();
    await expect(typeSelect).toHaveValue('Area');

    // 5. Switch to Point Text
    await typeSelect.selectOption('Point');
    await expect(typeSelect).toHaveValue('Point');

    // 6. Verify font size input appears for Point Text
    const fontSizeInput = page.getByLabel('Schriftgröße');
    await expect(fontSizeInput).toBeVisible();

    // 7. Type content and verify auto-expansion (heuristic check via width change)
    const initialBox = await frame.boundingBox();
    const textarea = page.locator('.properties textarea');
    await textarea.fill('This is a longer headline for testing');
    
    // We wait for the Svelte update
    await page.waitForTimeout(100);
    const expandedBox = await frame.boundingBox();
    
    expect(expandedBox!.width).toBeGreaterThan(initialBox!.width);
  });
});
