import { test, expect } from '@playwright/test'
import { mockAllApis } from '../fixtures/api-mock'

test.describe('Navigation', () => {
  test('app bar shows Harvex title', async ({ page }) => {
    await mockAllApis(page)
    await page.goto('/')

    await expect(page.locator('.v-app-bar').getByText('Harvex')).toBeVisible()
  })

  test('hamburger menu opens navigation drawer', async ({ page }) => {
    await mockAllApis(page)
    await page.goto('/')

    // The nav icon button is the first button in the app bar
    await page.locator('.v-app-bar button').first().click()

    const drawer = page.locator('.v-navigation-drawer')
    await expect(drawer).toBeVisible()
    await expect(drawer.getByText('Dashboard')).toBeVisible()
    await expect(drawer.getByText('Upload')).toBeVisible()
    await expect(drawer.getByText('Settings')).toBeVisible()
  })

  test('drawer link navigates to upload', async ({ page }) => {
    await mockAllApis(page)
    await page.goto('/')

    await page.locator('.v-app-bar button').first().click()
    await page.locator('.v-navigation-drawer .v-list-item', { hasText: 'Upload' }).click()
    await expect(page).toHaveURL('/upload')
  })

  test('drawer link navigates to settings', async ({ page }) => {
    await mockAllApis(page)
    await page.goto('/')

    await page.locator('.v-app-bar button').first().click()
    await page.locator('.v-navigation-drawer .v-list-item', { hasText: 'Settings' }).click()
    await expect(page).toHaveURL('/settings')
  })

  test('drawer link navigates to dashboard', async ({ page }) => {
    await mockAllApis(page)
    await page.goto('/settings')

    // Wait for settings page to finish loading model info
    await page.waitForLoadState('networkidle')

    await page.locator('.v-app-bar button').first().click()
    // Use force: true to bypass Vuetify drawer animation viewport issues
    await page.locator('.v-navigation-drawer .v-list-item', { hasText: 'Dashboard' }).click({ force: true })
    await expect(page).toHaveURL('/')
  })

  test('theme toggle button exists in app bar', async ({ page }) => {
    await mockAllApis(page)
    await page.goto('/')

    // Theme toggle icon (sun or moon) should be in the app bar
    const themeBtn = page.locator('.v-app-bar .v-btn').filter({ has: page.locator('.mdi-weather-sunny, .mdi-weather-night') })
    await expect(themeBtn).toBeVisible()
  })

  test('theme toggle switches between light and dark', async ({ page }) => {
    await mockAllApis(page)
    await page.goto('/')

    // Get initial theme state from the v-app element
    const vApp = page.locator('.v-application')
    const initialClasses = await vApp.getAttribute('class') || ''
    const wasLight = initialClasses.includes('v-theme--light')

    // Click theme toggle
    const themeBtn = page.locator('.v-app-bar .v-btn').filter({ has: page.locator('.mdi-weather-sunny, .mdi-weather-night') })
    await themeBtn.click()

    // Theme should have changed
    if (wasLight) {
      await expect(vApp).toHaveClass(/v-theme--dark/)
    } else {
      await expect(vApp).toHaveClass(/v-theme--light/)
    }
  })

  test('upload icon in app bar navigates to upload', async ({ page }) => {
    await mockAllApis(page)
    await page.goto('/')

    // The upload button in app bar has mdi-upload icon
    const uploadBtn = page.locator('.v-app-bar .v-btn').filter({ has: page.locator('.mdi-upload') })
    await uploadBtn.click()
    await expect(page).toHaveURL('/upload')
  })

  test('clicking Harvex title navigates to dashboard', async ({ page }) => {
    await mockAllApis(page)
    await page.goto('/settings')

    await page.locator('.v-app-bar').getByText('Harvex').click()
    await expect(page).toHaveURL('/')
  })
})
