import { test, expect } from '@playwright/test';

test.describe('Content Mode Persistence Regression', () => {
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

  test('Reproduction: Content mode must reset when selecting a different frame after tool switch', async ({ page }) => {
    const pageElement = page.locator('.page').first();
    await pageElement.waitFor({ state: 'visible' });

    // 1. Create an existing frame (Frame A) using the Frame Tool (F)
    await page.click('.toolbar button:has-text("F")');
    await pageElement.evaluate((el) => {
      const down = new MouseEvent('mousedown', { bubbles: true, clientX: 100, clientY: 100 });
      el.dispatchEvent(down);
      const move = new MouseEvent('mousemove', { bubbles: true, clientX: 200, clientY: 200 });
      window.dispatchEvent(move);
      const up = new MouseEvent('mouseup', { bubbles: true });
      window.dispatchEvent(up);
    });
    await page.keyboard.press('Escape'); // Deselect

    // 2. Switch to Text Tool and create Frame B (Area Text)
    await page.click('.toolbar button:has-text("T")');
    await pageElement.evaluate((el) => {
      const down = new MouseEvent('mousedown', { bubbles: true, clientX: 400, clientY: 400 });
      el.dispatchEvent(down);
      const move = new MouseEvent('mousemove', { bubbles: true, clientX: 500, clientY: 550 });
      window.dispatchEvent(move);
      const up = new MouseEvent('mouseup', { bubbles: true });
      window.dispatchEvent(up);
    });

    // Area text creation does NOT enter content mode automatically. 
    // We explicitly trigger content mode for testing via evaluate.
    const frames = page.locator('.frame-container');
    await expect(frames).toHaveCount(2);
    
    const frameB = frames.nth(1);
    await frameB.locator('.frame-content').evaluate((el) => {
      const dbl = new MouseEvent('dblclick', { bubbles: true });
      el.dispatchEvent(dbl);
    });
    
    // Verify Selection Overlay (now decoupled) is in content mode
    const overlay = page.locator('.selection-overlay');
    await expect(overlay).toBeVisible();
    await expect(overlay).toHaveClass(/content-mode/);

    // 3. Switch to Move Tool (V)
    await page.click('.toolbar button:has-text("V")');

    // 4. Select Frame A
    const frameA = frames.nth(0);
    console.log('Selecting Frame A with Move Tool...');
    await frameA.locator('.frame-content').evaluate((el) => {
      const down = new MouseEvent('mousedown', { bubbles: true, clientX: 150, clientY: 150 });
      el.dispatchEvent(down);
    });

    // Assert: Selection Overlay for Frame A should NOT be in content mode
    await expect(overlay).toBeVisible();
    await expect(overlay).not.toHaveClass(/content-mode/);
  });
});
