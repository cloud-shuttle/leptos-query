import { test, expect } from '@playwright/test';

/**
 * TDD Tests for Demo Application Requirements
 * 
 * These tests define what a working demo application should do.
 * We'll implement the demo to make these tests pass.
 */

test.describe('Demo Application Requirements', () => {
  test.beforeEach(async ({ page }) => {
    // Navigate to the demo app
    await page.goto('/');
    
    // Wait for the app to load
    await page.waitForSelector('body', { timeout: 10000 });
  });

  test('should load without JavaScript errors', async ({ page }) => {
    // Listen for console errors
    const errors: string[] = [];
    page.on('console', msg => {
      if (msg.type() === 'error') {
        errors.push(msg.text());
      }
    });
    
    // Wait for the app to fully load
    await page.waitForTimeout(2000);
    
    // Check that there are no JavaScript errors
    expect(errors).toHaveLength(0);
  });

  test('should display a basic user interface', async ({ page }) => {
    // Check that the page has a title
    const title = await page.title();
    expect(title).toBeTruthy();
    
    // Check that the page has some content
    const bodyText = await page.textContent('body');
    expect(bodyText).toBeTruthy();
    expect(bodyText!.length).toBeGreaterThan(0);
  });

  test('should demonstrate basic query functionality', async ({ page }) => {
    // Look for specific text that indicates query functionality
    const hasQueryText = await page.textContent('body');
    const hasQueryTerms = /query|fetch|data|user/i.test(hasQueryText || '');
    expect(hasQueryTerms).toBe(true);
  });

  test('should be responsive on different screen sizes', async ({ page }) => {
    // Test mobile viewport
    await page.setViewportSize({ width: 375, height: 667 });
    const mobileContent = await page.textContent('body');
    expect(mobileContent).toBeTruthy();
    
    // Test desktop viewport
    await page.setViewportSize({ width: 1920, height: 1080 });
    const desktopContent = await page.textContent('body');
    expect(desktopContent).toBeTruthy();
  });

  test('should handle user interactions without crashing', async ({ page }) => {
    // Try to click on any clickable elements
    const clickableElements = await page.locator('button, a, input[type="button"], input[type="submit"]').count();
    
    if (clickableElements > 0) {
      // Click the first clickable element
      await page.locator('button, a, input[type="button"], input[type="submit"]').first().click();
      
      // Wait a bit to see if anything crashes
      await page.waitForTimeout(1000);
      
      // Check that the page is still responsive
      const bodyText = await page.textContent('body');
      expect(bodyText).toBeTruthy();
    }
  });

  test('should demonstrate leptos-query features', async ({ page }) => {
    // Look for text that indicates leptos-query features
    const pageContent = await page.textContent('body');
    
    // Check for common query-related terms
    const hasQueryTerms = /query|cache|fetch|data|loading|error/i.test(pageContent || '');
    expect(hasQueryTerms).toBe(true);
  });
});
