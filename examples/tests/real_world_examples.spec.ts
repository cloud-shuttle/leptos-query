import { test, expect } from '@playwright/test';

/**
 * TDD Tests for Real-World Examples
 * 
 * These tests define what real-world examples should demonstrate.
 * We'll implement examples to make these tests pass.
 */

test.describe('Real-World Examples Requirements', () => {
  test('should have a todo app example', async ({ page }) => {
    // Navigate to todo app example
    await page.goto('/examples/todo-app.html');
    
    // Check that the page loads
    await expect(page.locator('h1')).toContainText(/todo|task/i);
    
    // Check for basic todo functionality
    const hasInput = await page.locator('input[type="text"], textarea').count();
    expect(hasInput).toBeGreaterThan(0);
    
    const hasButton = await page.locator('button').count();
    expect(hasButton).toBeGreaterThan(0);
  });

  test('should have a blog app example', async ({ page }) => {
    // Navigate to blog app example
    await page.goto('/examples/blog-app.html');
    
    // Check that the page loads
    await expect(page.locator('h1')).toContainText(/blog|post/i);
    
    // Check for blog functionality
    const hasPosts = await page.locator('*').filter({ hasText: /post|article|blog/i }).count();
    expect(hasPosts).toBeGreaterThan(0);
  });

  test('should have a weather app example', async ({ page }) => {
    // Navigate to weather app example
    await page.goto('/examples/weather-app.html');
    
    // Check that the page loads
    await expect(page.locator('h1')).toContainText(/weather|forecast/i);
    
    // Check for weather functionality
    const hasWeatherData = await page.locator('*').filter({ hasText: /temperature|weather|forecast/i }).count();
    expect(hasWeatherData).toBeGreaterThan(0);
  });

  test('should demonstrate caching behavior', async ({ page }) => {
    // Navigate to any example
    await page.goto('/examples/todo-app.html');
    
    // Look for cache-related functionality
    const hasCacheText = await page.textContent('body');
    const hasCacheTerms = /cache|stale|fresh|refetch/i.test(hasCacheText || '');
    expect(hasCacheTerms).toBe(true);
  });

  test('should demonstrate error handling', async ({ page }) => {
    // Navigate to any example
    await page.goto('/examples/todo-app.html');
    
    // Look for error handling functionality
    const hasErrorText = await page.textContent('body');
    const hasErrorTerms = /error|retry|failed/i.test(hasErrorText || '');
    expect(hasErrorTerms).toBe(true);
  });

  test('should demonstrate loading states', async ({ page }) => {
    // Navigate to any example
    await page.goto('/examples/todo-app.html');
    
    // Look for loading state functionality
    const hasLoadingText = await page.textContent('body');
    const hasLoadingTerms = /loading|spinner|fetching/i.test(hasLoadingText || '');
    expect(hasLoadingTerms).toBe(true);
  });
});
