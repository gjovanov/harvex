import { test, expect } from '@playwright/test'
import { mockAllApis } from '../fixtures/api-mock'

test.describe('Settings', () => {
  test('renders settings page with model config', async ({ page }) => {
    await mockAllApis(page)
    await page.goto('/settings')

    await expect(page.locator('h1', { hasText: 'Settings' })).toBeVisible()
    await expect(page.locator('.v-card-title', { hasText: 'LLM Model' })).toBeVisible()
    await expect(page.getByLabel('API URL')).toBeVisible()
    await expect(page.getByLabel('API Key')).toBeVisible()
    await expect(page.getByLabel('Max Tokens')).toBeVisible()
    await expect(page.getByLabel('Context Size')).toBeVisible()
  })

  test('loads current model settings', async ({ page }) => {
    await mockAllApis(page)
    await page.goto('/settings')

    const apiUrl = page.getByLabel('API URL')
    await expect(apiUrl).toHaveValue('http://localhost:11434/v1')
  })

  test('health check section exists', async ({ page }) => {
    await mockAllApis(page)
    await page.goto('/settings')

    await expect(page.locator('.v-card-title', { hasText: 'API Health' })).toBeVisible()
    await expect(page.getByRole('button', { name: 'Check Connection' })).toBeVisible()
  })

  test('check connection shows reachable status', async ({ page }) => {
    await mockAllApis(page)
    await page.goto('/settings')

    await page.getByRole('button', { name: 'Check Connection' }).click()

    await expect(page.getByText('API is reachable')).toBeVisible()
  })

  test('check connection shows unreachable status', async ({ page }) => {
    await mockAllApis(page, { modelHealth: { model_name: 'qwen2.5-vl:7b', api_url: 'http://localhost:11434/v1', reachable: false } })
    await page.goto('/settings')

    await page.getByRole('button', { name: 'Check Connection' }).click()

    await expect(page.getByText('API is not reachable')).toBeVisible()
  })

  test('available models section has refresh button', async ({ page }) => {
    await mockAllApis(page)
    await page.goto('/settings')

    await expect(page.locator('.v-card-title', { hasText: 'Available Models' })).toBeVisible()
    await expect(page.getByRole('button', { name: 'Refresh List' })).toBeVisible()
  })

  test('refresh list shows available models', async ({ page }) => {
    await mockAllApis(page)
    await page.goto('/settings')

    await page.getByRole('button', { name: 'Refresh List' }).click()

    // Use the list items specifically to avoid matching the v-select
    const modelsList = page.locator('.v-card', { hasText: 'Available Models' })
    await expect(modelsList.locator('.v-list-item-title', { hasText: 'qwen2.5-vl:7b' })).toBeVisible()
    await expect(modelsList.locator('.v-list-item-title', { hasText: 'qwen2.5-vl:3b' })).toBeVisible()
  })

  test('save settings button exists', async ({ page }) => {
    await mockAllApis(page)
    await page.goto('/settings')

    await expect(page.getByRole('button', { name: 'Save Settings' })).toBeVisible()
  })

  test('back button navigates to dashboard', async ({ page }) => {
    await mockAllApis(page)
    await page.goto('/settings')

    await page.locator('.v-btn').filter({ has: page.locator('.mdi-arrow-left') }).first().click()
    await expect(page).toHaveURL('/')
  })
})
