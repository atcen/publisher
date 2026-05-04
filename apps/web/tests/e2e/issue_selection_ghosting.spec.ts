import { test, expect } from '@playwright/test';

test.describe('Selection Ghosting Regression', () => {
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

  test('Reproduction: Handles must disappear from previous frames on new selection', async ({ page }) => {
    const pageElement = page.locator('.page').first();
    await pageElement.waitFor({ state: 'visible' });

    // 1. Create Frame A (Point Text)
    await page.click('button[title*="Text (T)"]');
    await pageElement.evaluate((el) => {
      const down = new MouseEvent('mousedown', { bubbles: true, clientX: 200, clientY: 200 });
      el.dispatchEvent(down);
      const up = new MouseEvent('mouseup', { bubbles: true });
      window.dispatchEvent(up);
    });

    // 2. Create 5 Frames (total 5)
    for (let i = 0; i < 5; i++) {
      await pageElement.evaluate((el, i) => {
        const x = 50 + i * 110;
        const y = 50 + i * 110;
        const down = new MouseEvent('mousedown', { bubbles: true, clientX: x, clientY: y });
        el.dispatchEvent(down);
        const up = new MouseEvent('mouseup', { bubbles: true });
        window.dispatchEvent(up);
      }, i);
      await page.keyboard.press('Escape'); // Exit content mode
    }

    // 3. Click each frame in sequence
    const frames = page.locator('.frame-container');
    const count = await frames.count();
    expect(count).toBe(6);

    for (let i = 0; i < count; i++) {
      console.log(`Selecting Frame ${i}...`);
      await frames.nth(i).locator('.frame-content').evaluate((el) => {
        const down = new MouseEvent('mousedown', { bubbles: true });
        el.dispatchEvent(down);
      });

      // Verify ONLY this frame has handles
      // Since they are decoupled, we just check that there is exactly ONE selection overlay
      // and it matches our expectations if needed.
      const totalOverlays = await page.locator('.selection-overlay').count();
      expect(totalOverlays).toBe(1);
    }

    // 5. Deselect all (click empty workspace)
    console.log('Deselecting all...');
    await page.locator('.workspace-container').evaluate((el) => {
       const down = new MouseEvent('mousedown', { bubbles: true, clientX: 10, clientY: 10 });
       el.dispatchEvent(down);
    });

    // Assert no handles visible at all
    await expect(page.locator('.selection-overlay')).not.toBeVisible();
  });
});
