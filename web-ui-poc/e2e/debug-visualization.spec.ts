import { test, expect } from '@playwright/test';

test.describe('Debug 3D Visualization', () => {
  test('capture console logs and screenshot', async ({ page }) => {
    // Store all console messages
    const consoleMessages: string[] = [];

    // Capture all console output
    page.on('console', msg => {
      const text = msg.text();
      consoleMessages.push(`[${msg.type()}] ${text}`);

      // Print to test output for immediate visibility
      console.log(`🔍 Console [${msg.type()}]: ${text}`);
    });

    // Capture errors
    page.on('pageerror', error => {
      console.error('❌ Page Error:', error.message);
      consoleMessages.push(`[ERROR] ${error.message}`);
    });

    // Navigate to the page
    console.log('\n📡 Navigating to http://localhost:3000');
    await page.goto('http://localhost:3000', {
      waitUntil: 'networkidle',
      timeout: 30000
    });

    console.log('✅ Page loaded, waiting for 3D scene to initialize...');

    // Wait for the canvas to be present (3D scene)
    await page.waitForSelector('canvas', { timeout: 10000 });
    console.log('✅ Canvas found');

    // Wait additional time for Three.js to initialize and render
    await page.waitForTimeout(5000);
    console.log('✅ Waited 5 seconds for rendering');

    // Check if Parseltongue API is accessible
    try {
      const apiResponse = await page.goto('http://localhost:7777/server-health-check-status');
      if (apiResponse && apiResponse.ok()) {
        console.log('✅ Parseltongue API is accessible');
        const apiStatus = await apiResponse.json();
        console.log('📊 API Status:', JSON.stringify(apiStatus, null, 2));
      } else {
        console.error('❌ Parseltongue API returned error:', apiResponse?.status());
      }
    } catch (error) {
      console.error('❌ Cannot reach Parseltongue API:', error);
    }

    // Take a full-page screenshot
    const screenshotPath = 'test-results/debug-screenshot.png';
    await page.screenshot({
      path: screenshotPath,
      fullPage: true
    });
    console.log(`📸 Screenshot saved to: ${screenshotPath}`);

    // Check for key console messages
    const dependencyMessages = consoleMessages.filter(msg =>
      msg.includes('Loading dependencies') ||
      msg.includes('Loaded') && msg.includes('dependency') ||
      msg.includes('Rendered') && msg.includes('arc')
    );

    console.log('\n🔍 Key Dependency Messages Found:');
    if (dependencyMessages.length > 0) {
      dependencyMessages.forEach(msg => console.log(`  ${msg}`));
    } else {
      console.log('  ⚠️  No dependency-related messages found!');
    }

    // Check for errors
    const errorMessages = consoleMessages.filter(msg =>
      msg.includes('[ERROR]') || msg.includes('[error]')
    );

    if (errorMessages.length > 0) {
      console.log('\n❌ Errors Found:');
      errorMessages.forEach(msg => console.log(`  ${msg}`));
    }

    // Analyze what we found
    console.log('\n📊 Analysis:');
    console.log(`  Total console messages: ${consoleMessages.length}`);
    console.log(`  Dependency-related: ${dependencyMessages.length}`);
    console.log(`  Errors: ${errorMessages.length}`);

    // Check if the scene has any objects (basic Three.js check)
    const sceneStats = await page.evaluate(() => {
      const canvas = document.querySelector('canvas');
      if (!canvas) return { error: 'No canvas found' };

      // Try to get Three.js info from the window
      const scene = (window as any).scene;
      if (!scene) return { error: 'Scene not exposed to window' };

      return {
        objectCount: scene.children?.length || 0,
        hasChildren: !!scene.children
      };
    });

    console.log('\n🎮 Scene Stats:', JSON.stringify(sceneStats, null, 2));

    // Check canvas content
    const canvasInfo = await page.evaluate(() => {
      const canvas = document.querySelector('canvas');
      if (!canvas) return { error: 'No canvas' };

      return {
        width: canvas.width,
        height: canvas.height,
        style: canvas.getAttribute('style'),
        classes: canvas.getAttribute('class')
      };
    });

    console.log('\n🖼️  Canvas Info:', JSON.stringify(canvasInfo, null, 2));

    // Log all console messages at the end for easy reference
    console.log('\n📝 Complete Console Log:');
    consoleMessages.forEach((msg, i) => {
      console.log(`  ${i + 1}. ${msg}`);
    });

    // Assertions to help debug
    expect(consoleMessages.length, 'Should have some console output').toBeGreaterThan(0);
    expect(await page.locator('canvas').count(), 'Should have a canvas element').toBe(1);

    // Note: We're not asserting dependency messages exist because that's what we're debugging
  });
});
