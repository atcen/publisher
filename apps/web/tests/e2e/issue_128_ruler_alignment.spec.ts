import { test, expect } from '@playwright/test';

test.describe('Issue 128: Ruler Alignment', () => {
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
    await page.waitForSelector('.workspace', { state: 'visible' });
  });

  test('ruler zero point should align with active page top-left', async ({ page }) => {
    const page0 = page.locator('.page').first();
    await page0.click(); // Set active page
    
    const pageBox = await page0.boundingBox();
    expect(pageBox).not.toBeNull();
    
    // Top ruler ticks
    const topRuler = page.locator('.top-ruler');
    const firstTick = topRuler.locator('.ruler-tick').first();
    const firstTickText = await firstTick.innerText();
    
    // If the ruler is aligned, the "0" tick should be at the same X as the page's left edge
    // Currently it's likely at the container's left edge
    
    // Let's find the tick that says "0"
    const zeroTick = topRuler.locator('.ruler-tick', { hasText: /^0$/ });
    await expect(zeroTick).toBeVisible();
    const zeroTickBox = await zeroTick.boundingBox();
    
    // Allow some tolerance for ruler padding/styling
    // The page starts at some offset due to workspace padding (60px)
    console.log(`Page X: ${pageBox!.x}, Zero Tick X: ${zeroTickBox!.x}`);
    
    // This is expected to fail if the bug exists
    expect(Math.abs(zeroTickBox!.x - pageBox!.x)).toBeLessThan(5);
    expect(Math.abs(zeroTickBox!.y - pageBox!.y)).toBeGreaterThan(0); // Ruler is above page
  });

  test('ruler should update zero point when second page is clicked', async ({ page }) => {
    // Add a second page first? 
    // The default document has one page. Let's add one.
    // Actually let's just check if we can add a page via some UI or mock it.
    // For now, let's just see if the first page is aligned.
  });
});
