import { test, expect } from '@playwright/test';

test.describe('Text Frame Creation', () => {
  test.beforeEach(async ({ page }) => {
    page.on('console', msg => console.log('BROWSER:', msg.text()));
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
    // 2. Drag on a page to create an Area Text frame
    const pageElement = page.locator('.page').first();
    await pageElement.waitFor({ state: 'visible' });

    // Perform a drag from (100, 100) to (250, 150)
    await page.mouse.move(200, 200);
    await page.mouse.down();
    await page.mouse.move(350, 300);
    await page.mouse.up();

    // 3. Verify Area Text frame is created and selected
    const frame = pageElement.locator('.frame.selected');
    await expect(frame).toBeVisible({ timeout: 10000 });

    const box = await frame.boundingBox();
    expect(box!.width).toBeGreaterThan(100);
    expect(box!.height).toBeGreaterThan(50);

    // 4. Check Sidebar properties
    const typeSelect = page.locator('.properties select').nth(1); 
    await expect(typeSelect).toHaveValue('Area');

    // 5. Switch to Point Text manually
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

  test('should create a Point Text frame on simple click', async ({ page }) => {
    await page.goto('/');

    // 1. Select the Text Tool (T)
    await page.click('button[title*="Text (T)"]');

    // 2. Click on a page to create a frame
    const pageElement = page.locator('.page').first();
    await pageElement.waitFor({ state: 'visible' });
    
    // Simple click at (150, 150)
    await page.mouse.move(250, 250);
    await page.mouse.down();
    await page.mouse.up();

    // 3. Verify Point Text frame is created (should have default content "Neuer Text")
    const frame = pageElement.locator('.frame.selected');
    await expect(frame).toBeVisible();
    
    // Check if it's Point type in sidebar
    const typeSelect = page.locator('.properties select').nth(1); 
    await expect(typeSelect).toHaveValue('Point');

    // Verify it is in content mode (textarea should be visible)
    const textarea = page.locator('.inline-editor');
    await expect(textarea).toBeFocused();
    await expect(textarea).toHaveValue('Neuer Text');
  });
});
