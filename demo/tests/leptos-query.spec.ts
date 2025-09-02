import { test, expect } from '@playwright/test';

/**
 * Comprehensive E2E tests for the Leptos Query demo application
 * Tests cover data fetching, caching, mutations, and error handling
 */

test.describe('Leptos Query Demo Application', () => {
  test.beforeEach(async ({ page }) => {
    // Navigate to the demo app before each test
    await page.goto('/');
    
    // Wait for the app to load
    await page.waitForSelector('[data-testid="app-loaded"]', { timeout: 10000 });
  });

  test.describe('Basic Functionality', () => {
    test('should load the application successfully', async ({ page }) => {
      // Check that the main app container is visible
      await expect(page.locator('[data-testid="app-container"]')).toBeVisible();
      
      // Check that the title is displayed
      await expect(page.locator('h1')).toContainText('Leptos Query Demo');
    });

    test('should display navigation menu', async ({ page }) => {
      // Check that navigation elements are present
      await expect(page.locator('[data-testid="nav-menu"]')).toBeVisible();
      
      // Check for expected navigation items
      const navItems = ['Basic Usage', 'Advanced Usage', 'Caching', 'Mutations'];
      for (const item of navItems) {
        await expect(page.locator(`[data-testid="nav-${item.toLowerCase().replace(' ', '-')}"]`)).toBeVisible();
      }
    });
  });

  test.describe('Data Fetching', () => {
    test('should fetch and display user data', async ({ page }) => {
      // Navigate to basic usage section
      await page.click('[data-testid="nav-basic-usage"]');
      
      // Wait for the fetch button to be visible
      await expect(page.locator('[data-testid="fetch-user-btn"]')).toBeVisible();
      
      // Click fetch button
      await page.click('[data-testid="fetch-user-btn"]');
      
      // Wait for loading state
      await expect(page.locator('[data-testid="loading-indicator"]')).toBeVisible();
      
      // Wait for data to load
      await expect(page.locator('[data-testid="user-data"]')).toBeVisible({ timeout: 10000 });
      
      // Verify user data is displayed
      await expect(page.locator('[data-testid="user-name"]')).toContainText('John Doe');
      await expect(page.locator('[data-testid="user-email"]')).toContainText('john@example.com');
    });

    test('should handle fetch errors gracefully', async ({ page }) => {
      // Navigate to error handling section
      await page.click('[data-testid="nav-error-handling"]');
      
      // Trigger an error by clicking error button
      await page.click('[data-testid="trigger-error-btn"]');
      
      // Wait for error state
      await expect(page.locator('[data-testid="error-message"]')).toBeVisible();
      
      // Verify error message
      await expect(page.locator('[data-testid="error-message"]')).toContainText('Failed to fetch');
    });

    test('should show loading states during fetch', async ({ page }) => {
      // Navigate to basic usage
      await page.click('[data-testid="nav-basic-usage"]');
      
      // Click fetch button
      await page.click('[data-testid="fetch-user-btn"]');
      
      // Verify loading indicator appears
      await expect(page.locator('[data-testid="loading-indicator"]')).toBeVisible();
      
      // Wait for loading to complete
      await expect(page.locator('[data-testid="loading-indicator"]')).not.toBeVisible({ timeout: 10000 });
    });
  });

  test.describe('Caching Behavior', () => {
    test('should cache data and avoid refetching', async ({ page }) => {
      // Navigate to caching section
      await page.click('[data-testid="nav-caching"]');
      
      // First fetch
      await page.click('[data-testid="fetch-cached-btn"]');
      await expect(page.locator('[data-testid="cached-data"]')).toBeVisible({ timeout: 10000 });
      
      // Note the timestamp
      const firstTimestamp = await page.locator('[data-testid="fetch-timestamp"]').textContent();
      
      // Second fetch (should use cache)
      await page.click('[data-testid="fetch-cached-btn"]');
      
      // Verify timestamp hasn't changed (cache hit)
      const secondTimestamp = await page.locator('[data-testid="fetch-timestamp"]').textContent();
      expect(secondTimestamp).toBe(firstTimestamp);
      
      // Verify cache hit indicator
      await expect(page.locator('[data-testid="cache-hit-indicator"]')).toBeVisible();
    });

    test('should respect stale time settings', async ({ page }) => {
      // Navigate to caching section
      await page.click('[data-testid="nav-caching"]');
      
      // Set short stale time
      await page.fill('[data-testid="stale-time-input"]', '1000');
      
      // Fetch data
      await page.click('[data-testid="fetch-cached-btn"]');
      await expect(page.locator('[data-testid="cached-data"]')).toBeVisible({ timeout: 10000 });
      
      // Wait for data to become stale
      await page.waitForTimeout(1500);
      
      // Fetch again (should refetch due to stale data)
      await page.click('[data-testid="fetch-cached-btn"]');
      
      // Verify refetch occurred
      await expect(page.locator('[data-testid="refetch-indicator"]')).toBeVisible();
    });
  });

  test.describe('Mutations', () => {
    test('should create new posts successfully', async ({ page }) => {
      // Navigate to mutations section
      await page.click('[data-testid="nav-mutations"]');
      
      // Fill in post form
      await page.fill('[data-testid="post-title-input"]', 'Test Post Title');
      await page.fill('[data-testid="post-content-input"]', 'This is a test post content');
      
      // Submit form
      await page.click('[data-testid="create-post-btn"]');
      
      // Wait for success message
      await expect(page.locator('[data-testid="success-message"]')).toBeVisible({ timeout: 10000 });
      
      // Verify post appears in list
      await expect(page.locator('[data-testid="post-list"]')).toContainText('Test Post Title');
    });

    test('should handle mutation errors', async ({ page }) => {
      // Navigate to mutations section
      await page.click('[data-testid="nav-mutations"]');
      
      // Try to create post with invalid data
      await page.fill('[data-testid="post-title-input"]', ''); // Empty title
      await page.fill('[data-testid="post-content-input"]', 'Content');
      
      // Submit form
      await page.click('[data-testid="create-post-btn"]');
      
      // Verify error message
      await expect(page.locator('[data-testid="error-message"]')).toBeVisible();
      await expect(page.locator('[data-testid="error-message"]')).toContainText('Title is required');
    });

    test('should show optimistic updates', async ({ page }) => {
      // Navigate to mutations section
      await page.click('[data-testid="nav-mutations"]');
      
      // Fill in post form
      await page.fill('[data-testid="post-title-input"]', 'Optimistic Post');
      await page.fill('[data-testid="post-content-input"]', 'Optimistic content');
      
      // Submit form
      await page.click('[data-testid="create-post-btn"]');
      
      // Verify optimistic update appears immediately
      await expect(page.locator('[data-testid="post-list"]')).toContainText('Optimistic Post');
      
      // Verify optimistic indicator
      await expect(page.locator('[data-testid="optimistic-indicator"]')).toBeVisible();
      
      // Wait for mutation to complete
      await expect(page.locator('[data-testid="optimistic-indicator"]')).not.toBeVisible({ timeout: 10000 });
    });
  });

  test.describe('Advanced Features', () => {
    test('should handle infinite queries', async ({ page }) => {
      // Navigate to advanced usage
      await page.click('[data-testid="nav-advanced-usage"]');
      
      // Scroll to load more items
      await page.evaluate(() => window.scrollTo(0, document.body.scrollHeight));
      
      // Wait for more items to load
      await expect(page.locator('[data-testid="infinite-item"]')).toHaveCount(20, { timeout: 10000 });
      
      // Scroll again to load more
      await page.evaluate(() => window.scrollTo(0, document.body.scrollHeight));
      
      // Verify more items loaded
      await expect(page.locator('[data-testid="infinite-item"]')).toHaveCount(40, { timeout: 10000 });
    });

    test('should handle query invalidation', async ({ page }) => {
      // Navigate to advanced usage
      await page.click('[data-testid="nav-advanced-usage"]');
      
      // Fetch initial data
      await page.click('[data-testid="fetch-data-btn"]');
      await expect(page.locator('[data-testid="data-display"]')).toBeVisible({ timeout: 10000 });
      
      // Invalidate queries
      await page.click('[data-testid="invalidate-btn"]');
      
      // Verify data is marked as stale
      await expect(page.locator('[data-testid="stale-indicator"]')).toBeVisible();
      
      // Fetch again to refresh
      await page.click('[data-testid="fetch-data-btn"]');
      
      // Verify fresh data
      await expect(page.locator('[data-testid="fresh-data-indicator"]')).toBeVisible();
    });

    test('should handle background refetching', async ({ page }) => {
      // Navigate to advanced usage
      await page.click('[data-testid="nav-advanced-usage"]');
      
      // Enable background refetching
      await page.click('[data-testid="background-refetch-toggle"]');
      
      // Fetch data
      await page.click('[data-testid="fetch-data-btn"]');
      await expect(page.locator('[data-testid="data-display"]')).toBeVisible({ timeout: 10000 });
      
      // Focus window to trigger background refetch
      await page.bringToFront();
      
      // Verify background refetch indicator
      await expect(page.locator('[data-testid="background-refetch-indicator"]')).toBeVisible();
    });
  });

  test.describe('Performance and Responsiveness', () => {
    test('should handle rapid user interactions', async ({ page }) => {
      // Navigate to basic usage
      await page.click('[data-testid="nav-basic-usage"]');
      
      // Rapidly click fetch button multiple times
      for (let i = 0; i < 5; i++) {
        await page.click('[data-testid="fetch-user-btn"]');
        await page.waitForTimeout(100);
      }
      
      // Verify app remains responsive
      await expect(page.locator('[data-testid="app-container"]')).toBeVisible();
      
      // Verify no duplicate requests (deduplication working)
      await expect(page.locator('[data-testid="request-count"]')).toContainText('1');
    });

    test('should handle large data sets efficiently', async ({ page }) => {
      // Navigate to performance section
      await page.click('[data-testid="nav-performance"]');
      
      // Load large dataset
      await page.click('[data-testid="load-large-data-btn"]');
      
      // Wait for data to load
      await expect(page.locator('[data-testid="large-data-list"]')).toBeVisible({ timeout: 15000 });
      
      // Verify virtualization is working (only visible items rendered)
      const visibleItems = await page.locator('[data-testid="virtualized-item"]').count();
      expect(visibleItems).toBeLessThan(1000); // Should be much less than total
      
      // Verify scrolling performance
      await page.evaluate(() => {
        const start = performance.now();
        window.scrollTo(0, 10000);
        return performance.now() - start;
      });
      
      // Should be fast (less than 100ms)
      const scrollTime = await page.evaluate(() => {
        const start = performance.now();
        window.scrollTo(0, 10000);
        return performance.now() - start;
      });
      
      expect(scrollTime).toBeLessThan(100);
    });
  });

  test.describe('Error Boundaries and Recovery', () => {
    test('should recover from component errors', async ({ page }) => {
      // Navigate to error handling
      await page.click('[data-testid="nav-error-handling"]');
      
      // Trigger component error
      await page.click('[data-testid="trigger-component-error"]');
      
      // Verify error boundary catches it
      await expect(page.locator('[data-testid="error-boundary"]')).toBeVisible();
      
      // Try to recover
      await page.click('[data-testid="recover-btn"]');
      
      // Verify app is back to normal
      await expect(page.locator('[data-testid="app-container"]')).toBeVisible();
      await expect(page.locator('[data-testid="error-boundary"]')).not.toBeVisible();
    });

    test('should handle network failures gracefully', async ({ page }) => {
      // Navigate to error handling
      await page.click('[data-testid="nav-error-handling"]');
      
      // Simulate network failure
      await page.click('[data-testid="simulate-network-failure"]');
      
      // Verify offline indicator
      await expect(page.locator('[data-testid="offline-indicator"]')).toBeVisible();
      
      // Try to fetch data
      await page.click('[data-testid="fetch-user-btn"]');
      
      // Verify offline error message
      await expect(page.locator('[data-testid="offline-error"]')).toBeVisible();
      
      // Restore network
      await page.click('[data-testid="restore-network"]');
      
      // Verify offline indicator disappears
      await expect(page.locator('[data-testid="offline-indicator"]')).not.toBeVisible();
    });
  });

  test.describe('Accessibility', () => {
    test('should have proper ARIA labels', async ({ page }) => {
      // Check for proper ARIA labels on interactive elements
      await expect(page.locator('[data-testid="fetch-user-btn"]')).toHaveAttribute('aria-label');
      await expect(page.locator('[data-testid="post-title-input"]')).toHaveAttribute('aria-label');
      
      // Check for proper roles
      await expect(page.locator('[data-testid="nav-menu"]')).toHaveAttribute('role', 'navigation');
      await expect(page.locator('[data-testid="post-list"]')).toHaveAttribute('role', 'list');
    });

    test('should support keyboard navigation', async ({ page }) => {
      // Navigate to basic usage
      await page.click('[data-testid="nav-basic-usage"]');
      
      // Focus on fetch button
      await page.keyboard.press('Tab');
      await expect(page.locator('[data-testid="fetch-user-btn"]')).toBeFocused();
      
      // Activate with Enter key
      await page.keyboard.press('Enter');
      
      // Verify action occurred
      await expect(page.locator('[data-testid="loading-indicator"]')).toBeVisible();
    });

    test('should have proper focus management', async ({ page }) => {
      // Navigate to mutations
      await page.click('[data-testid="nav-mutations"]');
      
      // Focus on title input
      await page.locator('[data-testid="post-title-input"]').focus();
      
      // Verify focus is visible
      await expect(page.locator('[data-testid="post-title-input"]')).toBeFocused();
      
      // Check focus outline
      const focusOutline = await page.locator('[data-testid="post-title-input"]').evaluate(el => 
        window.getComputedStyle(el).outline !== 'none'
      );
      expect(focusOutline).toBe(true);
    });
  });

  test.describe('Cross-browser Compatibility', () => {
    test('should work consistently across different viewport sizes', async ({ page }) => {
      // Test mobile viewport
      await page.setViewportSize({ width: 375, height: 667 });
      await expect(page.locator('[data-testid="app-container"]')).toBeVisible();
      
      // Test tablet viewport
      await page.setViewportSize({ width: 768, height: 1024 });
      await expect(page.locator('[data-testid="app-container"]')).toBeVisible();
      
      // Test desktop viewport
      await page.setViewportSize({ width: 1920, height: 1080 });
      await expect(page.locator('[data-testid="app-container"]')).toBeVisible();
    });

    test('should handle different screen densities', async ({ page }) => {
      // Test with high DPI
      await page.evaluate(() => {
        Object.defineProperty(window, 'devicePixelRatio', {
          get: () => 2,
          configurable: true
        });
      });
      
      // Verify app still renders correctly
      await expect(page.locator('[data-testid="app-container"]')).toBeVisible();
      
      // Reset DPI
      await page.evaluate(() => {
        Object.defineProperty(window, 'devicePixelRatio', {
          get: () => 1,
          configurable: true
        });
      });
    });
  });
});
