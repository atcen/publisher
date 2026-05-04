import { test, expect } from '@playwright/test';

test.describe('Text Frame Comprehensive Lifecycle', () => {
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
    // Wait for the app to be fully loaded (check for a key element)
    await page.waitForSelector('.toolbar', { state: 'visible' });
  });

  test('Professional Lifecycle: Create, Type, Resize', async ({ page }) => {
    // 1. Select Text Tool
    await page.click('button[title*="Text (T)"]');

    // 2. Dispatch Drag Event to create Area Text
    const pageElement = page.locator('.page').first();
    await pageElement.waitFor({ state: 'visible' });

    // We use evaluate to dispatch events directly, bypassing coordinate mapping issues
    await pageElement.evaluate((el) => {
      const down = new MouseEvent('mousedown', { bubbles: true, clientX: 300, clientY: 300 });
      el.dispatchEvent(down);
      
      const move = new MouseEvent('mousemove', { bubbles: true, clientX: 500, clientY: 450 });
      window.dispatchEvent(move);
      
      const up = new MouseEvent('mouseup', { bubbles: true });
      window.dispatchEvent(up);
    });

    // 3. Verify Frame Existence and Size
    const frame = page.locator('.frame-container.selected');
    await expect(frame).toBeVisible({ timeout: 5000 });
    
    // Check computed style for visibility (on the content inside)
    const content = frame.locator('.frame-content');
    const color = await content.evaluate((el) => window.getComputedStyle(el).color);
    console.log('COMPUTED COLOR:', color);
    expect(color).not.toBe('rgb(255, 255, 255)'); // Must not be white
    
    const box = await frame.boundingBox();
    expect(box!.width).toBeGreaterThan(100);

    // 4. Verify Content Mode and Typing
    const editor = frame.locator('textarea.inline-editor');
    await expect(editor).toBeVisible();
    await editor.fill('VISUAL PROOF TEXT');
    
    // Take a screenshot of the frame for manual/AI verification
    await frame.screenshot({ path: 'text-frame-proof.png' });
    console.log('Screenshot saved to text-frame-proof.png');
    
    await page.keyboard.press('Escape');
    await expect(frame).toContainText('VISUAL PROOF TEXT');

    // 5. Verify Point Text via Click
    await pageElement.evaluate((el) => {
      const down = new MouseEvent('mousedown', { bubbles: true, clientX: 600, clientY: 600 });
      el.dispatchEvent(down);
      const up = new MouseEvent('mouseup', { bubbles: true });
      window.dispatchEvent(up);
    });

    const pointFrame = page.locator('.frame-container.selected');
    await expect(pointFrame).toBeVisible();
    
    const typeSelect = page.locator('.properties select').nth(1);
    await expect(typeSelect).toHaveValue('Point');
  });

  test('Typographic Stability & Point Text Vertical Growth', async ({ page }) => {
    // 1. Create Point Text
    await page.click('button[title*="Text (T)"]');
    const pageElement = page.locator('.page').first();
    await pageElement.evaluate((el) => {
      const down = new MouseEvent('mousedown', { bubbles: true, clientX: 300, clientY: 300 });
      el.dispatchEvent(down);
      const up = new MouseEvent('mouseup', { bubbles: true });
      window.dispatchEvent(up);
    });

    const frame = page.locator('.frame-container.selected');
    const editor = frame.locator('textarea.inline-editor');
    
    // Check initial height matches line height (ceil(12px * 1.2) = 15px)
    const initialBox = await frame.boundingBox();
    expect(initialBox!.height).toBe(15); 

    // 2. Test Typographic Consistency (Editor vs Display)
    const getTypoStyles = async (loc: any) => {
      await loc.waitFor({ state: 'visible' });
      return await loc.evaluate((el: HTMLElement) => {
        const s = window.getComputedStyle(el);
        return {
          fontFamily: s.fontFamily,
          fontSize: s.fontSize,
          lineHeight: s.lineHeight,
          letterSpacing: s.letterSpacing,
          textAlign: s.textAlign,
          padding: s.padding,
          textRendering: s.textRendering,
          webkitFontSmoothing: s.webkitFontSmoothing
        };
      });
    };

    const editorStyles = await getTypoStyles(editor);
    
    await editor.fill('Consistency Check');
    await page.keyboard.press('Escape'); // Deselect/Exit content mode
    
    const wrapper = frame.locator('.text-content-wrapper');
    const wrapperStyles = await getTypoStyles(wrapper);
    
    // Assert perfect match
    expect(wrapperStyles.fontFamily).toBe(editorStyles.fontFamily);
    expect(wrapperStyles.fontSize).toBe(editorStyles.fontSize);
    expect(wrapperStyles.lineHeight).toBe(editorStyles.lineHeight);
    expect(wrapperStyles.textAlign).toBe(editorStyles.textAlign);
    expect(wrapperStyles.padding).toBe(editorStyles.padding);
    
    // 4. Test Trailing Newline (Enter Key)
    await page.keyboard.press('Enter'); // Re-enter content mode
    await expect(editor).toBeVisible();

    await editor.fill('Line 1');
    const boxBeforeEnter = await frame.boundingBox();
    
    await editor.press('Enter');
    const boxAfterEnter = await frame.boundingBox();
    
    // Height must increase immediately on newline
    expect(boxAfterEnter!.height).toBeGreaterThan(boxBeforeEnter!.height * 1.5);
    
    // Add more text to verify width still adapts
    await editor.type('Line 2 is much longer now to test width expansion');
    const wideBox = await frame.boundingBox();
    expect(wideBox!.width).toBeGreaterThan(boxAfterEnter!.width * 1.5);
  });
});
