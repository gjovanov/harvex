import { test, expect } from '@playwright/test'
import { mockAllApis } from '../fixtures/api-mock'
import { sampleDocuments } from '../fixtures/mock-data'

test.describe('Batch Detail', () => {
  test('shows batch name in header', async ({ page }) => {
    await mockAllApis(page)
    await page.goto('/batch/batch-001')

    await expect(page.getByText('January Invoices')).toBeVisible()
  })

  test('shows extractions tab by default', async ({ page }) => {
    await mockAllApis(page)
    await page.goto('/batch/batch-001')

    await expect(page.getByRole('tab', { name: /Extractions/ })).toBeVisible()
    // Extraction cards should be visible
    await expect(page.getByText('invoice').first()).toBeVisible()
  })

  test('shows extraction confidence', async ({ page }) => {
    await mockAllApis(page)
    await page.goto('/batch/batch-001')

    // At least one confidence value should be visible
    await expect(page.getByText('92%').or(page.getByText('0.92'))).toBeVisible()
  })

  test('switching to documents tab shows document table', async ({ page }) => {
    await mockAllApis(page)
    await page.goto('/batch/batch-001')

    await page.getByRole('tab', { name: /Documents/ }).click()

    // The documents tab window-item contains a table with document rows
    const docsTab = page.locator('.v-tabs-window-item').nth(1)

    for (const doc of sampleDocuments) {
      await expect(docsTab.getByText(doc.original_name)).toBeVisible()
    }
  })

  test('documents tab shows file sizes', async ({ page }) => {
    await mockAllApis(page)
    await page.goto('/batch/batch-001')

    await page.getByRole('tab', { name: /Documents/ }).click()

    // PDF is 245000 bytes ≈ 239.3 KB
    const docsTable = page.locator('.v-table')
    await expect(docsTable.getByText('KB').first()).toBeVisible()
  })

  test('export menu contains JSON, Excel, CSV options', async ({ page }) => {
    await mockAllApis(page)
    await page.goto('/batch/batch-001')

    await page.locator('.v-btn', { hasText: 'Export' }).click()

    await expect(page.getByText('JSON')).toBeVisible()
    await expect(page.getByText('Excel')).toBeVisible()
    await expect(page.getByText('CSV')).toBeVisible()
  })

  test('delete button opens confirmation', async ({ page }) => {
    await mockAllApis(page)
    await page.goto('/batch/batch-001')

    // The delete button in the button group with error color
    await page.locator('.v-btn-group .v-btn').last().click()

    await expect(page.getByText('Delete Batch')).toBeVisible()
    await expect(page.getByText('Are you sure')).toBeVisible()
  })

  test('back button navigates to dashboard', async ({ page }) => {
    await mockAllApis(page)
    await page.goto('/batch/batch-001')

    // The back button is the first icon button with arrow-left
    await page.locator('.v-btn').filter({ has: page.locator('.mdi-arrow-left') }).first().click()
    await expect(page).toHaveURL('/')
  })

  test('pending batch shows Process button', async ({ page }) => {
    await mockAllApis(page)
    await page.goto('/batch/batch-002')

    await expect(page.getByText('Process')).toBeVisible()
  })
})
