import { test, expect } from '@playwright/test';

/**
 * Simple E2E tests for the Leptos Query demo application
 * Tests the actual functionality present in the demo
 */

test.describe('Leptos Query Demo Application', () => {
  test.beforeEach(async ({ page }) => {
    // Navigate to the demo app before each test
    await page.goto('/');
    
    // Wait for the app to load
    await page.waitForSelector('h1', { timeout: 10000 });
  });

  test('should load the application successfully', async ({ page }) => {
    // Check that the main app container is visible
    await expect(page.locator('.app')).toBeVisible();
    
    // Check that the title is displayed
    await expect(page.locator('h1')).toContainText('Leptos Query Demo');
    
    // Check that the subtitle is displayed
    await expect(page.locator('p')).toContainText('A simple demonstration of leptos-query features');
  });

  test('should display user query demo section', async ({ page }) => {
    // Check that the demo section is visible
    await expect(page.locator('.demo-section')).toBeVisible();
    
    // Check that the section title is displayed
    await expect(page.locator('h2')).toContainText('User Query Demo');
    
    // Check that controls are present
    await expect(page.locator('.controls')).toBeVisible();
    await expect(page.locator('label')).toContainText('User ID:');
    await expect(page.locator('input[type="number"]')).toBeVisible();
  });

  test('should display query status information', async ({ page }) => {
    // Check that query status section is visible
    await expect(page.locator('.query-status')).toBeVisible();
    
    // Check that status information is displayed
    await expect(page.locator('h3')).toContainText('Query Status');
    await expect(page.locator('p')).toContainText('Loading:');
    await expect(page.locator('p')).toContainText('Stale:');
    await expect(page.locator('p')).toContainText('Error:');
  });

  test('should display user data section', async ({ page }) => {
    // Check that user data section is visible
    await expect(page.locator('.user-data')).toBeVisible();
    
    // Check that section title is displayed
    await expect(page.locator('h3')).toContainText('User Data');
    
    // Initially should show loading or initializing state
    const userDataSection = page.locator('.user-data');
    await expect(userDataSection).toBeVisible();
  });

  test('should display features list', async ({ page }) => {
    // Check that features section is visible
    await expect(page.locator('.features')).toBeVisible();
    
    // Check that features title is displayed
    await expect(page.locator('h3')).toContainText('Features Demonstrated');
    
    // Check that features list is present
    await expect(page.locator('ul')).toBeVisible();
    await expect(page.locator('li')).toHaveCount(5);
    
    // Check for specific features
    await expect(page.locator('li')).toContainText('Automatic caching');
    await expect(page.locator('li')).toContainText('Background updates');
    await expect(page.locator('li')).toContainText('Query key-based cache invalidation');
    await expect(page.locator('li')).toContainText('Built-in error handling');
    await expect(page.locator('li')).toContainText('Loading and stale state management');
  });

  test('should display footer', async ({ page }) => {
    // Check that footer is visible
    await expect(page.locator('.app-footer')).toBeVisible();
    
    // Check that footer content is displayed
    await expect(page.locator('p')).toContainText('Built with');
    await expect(page.locator('a')).toContainText('leptos-query');
    await expect(page.locator('a')).toHaveAttribute('href', 'https://github.com/cloud-shuttle/leptos-query');
    await expect(page.locator('a')).toHaveAttribute('target', '_blank');
  });

  test('should allow changing user ID', async ({ page }) => {
    // Get the input field
    const userInput = page.locator('input[type="number"]');
    
    // Check initial value
    await expect(userInput).toHaveValue('1');
    
    // Change the value
    await userInput.fill('5');
    await expect(userInput).toHaveValue('5');
    
    // Change to another value
    await userInput.fill('10');
    await expect(userInput).toHaveValue('10');
  });

  test('should handle invalid user ID input gracefully', async ({ page }) => {
    // Get the input field
    const userInput = page.locator('input[type="number"]');
    
    // Try to enter invalid value
    await userInput.fill('abc');
    
    // Should fallback to default value (1)
    await expect(userInput).toHaveValue('1');
  });

  test('should be responsive on different viewport sizes', async ({ page }) => {
    // Test mobile viewport
    await page.setViewportSize({ width: 375, height: 667 });
    await expect(page.locator('.app')).toBeVisible();
    
    // Test tablet viewport
    await page.setViewportSize({ width: 768, height: 1024 });
    await expect(page.locator('.app')).toBeVisible();
    
    // Test desktop viewport
    await page.setViewportSize({ width: 1920, height: 1080 });
    await expect(page.locator('.app')).toBeVisible();
  });

  test('should have proper page structure', async ({ page }) => {
    // Check for proper HTML structure
    await expect(page.locator('header.app-header')).toBeVisible();
    await expect(page.locator('main.app-main')).toBeVisible();
    await expect(page.locator('footer.app-footer')).toBeVisible();
    
    // Check for proper heading hierarchy
    await expect(page.locator('h1')).toHaveCount(1);
    await expect(page.locator('h2')).toHaveCount(1);
    await expect(page.locator('h3')).toHaveCount(3);
  });

  test('should load without JavaScript errors', async ({ page }) => {
    // Listen for console errors
    const errors: string[] = [];
    page.on('console', msg => {
      if (msg.type() === 'error') {
        errors.push(msg.text());
      }
    });
    
    // Navigate to the page
    await page.goto('/');
    
    // Wait for the app to load
    await page.waitForSelector('h1', { timeout: 10000 });
    
    // Check that there are no JavaScript errors
    expect(errors).toHaveLength(0);
  });
});
