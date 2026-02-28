import { test, expect } from '@playwright/test'
import { mockAllApis } from '../fixtures/api-mock'
import { sampleBatches } from '../fixtures/mock-data'

test.describe('Dashboard', () => {
  test('shows empty state when no batches exist', async ({ page }) => {
    await mockAllApis(page, { batches: [] })
    await page.goto('/')

    await expect(page.getByText('No batches yet')).toBeVisible()
    // The empty state "Upload Documents" button
    await expect(page.locator('.text-center .v-btn', { hasText: 'Upload Documents' })).toBeVisible()
  })

  test('displays batch cards', async ({ page }) => {
    await mockAllApis(page)
    await page.goto('/')

    for (const batch of sampleBatches) {
      await expect(page.getByText(batch.name)).toBeVisible()
    }
  })

  test('batch cards show status chip', async ({ page }) => {
    await mockAllApis(page)
    await page.goto('/')

    await expect(page.getByText('completed').first()).toBeVisible()
    await expect(page.getByText('pending').first()).toBeVisible()
    await expect(page.getByText('processing').first()).toBeVisible()
  })

  test('batch cards show file count', async ({ page }) => {
    await mockAllApis(page)
    await page.goto('/')

    await expect(page.getByText('5 files')).toBeVisible()
    await expect(page.getByText('3 files')).toBeVisible()
    await expect(page.getByText('10 files')).toBeVisible()
  })

  test('clicking a batch card navigates to detail', async ({ page }) => {
    await mockAllApis(page)
    await page.goto('/')

    await page.getByText('January Invoices').click()
    await expect(page).toHaveURL(/\/batch\/batch-001/)
  })

  test('pending batch shows Process button', async ({ page }) => {
    await mockAllApis(page)
    await page.goto('/')

    const pendingCard = page.locator('.v-card', { hasText: 'Bank Statements Q4' })
    await expect(pendingCard.getByText('Process')).toBeVisible()
  })

  test('delete button opens confirmation dialog', async ({ page }) => {
    await mockAllApis(page)
    await page.goto('/')

    const firstCard = page.locator('.v-card').first()
    await firstCard.locator('.mdi-delete').click()

    await expect(page.getByText('Delete Batch')).toBeVisible()
    await expect(page.getByText('Are you sure')).toBeVisible()
    await expect(page.getByRole('button', { name: 'Cancel' })).toBeVisible()
    await expect(page.getByRole('button', { name: 'Delete' })).toBeVisible()
  })

  test('cancel delete closes dialog', async ({ page }) => {
    await mockAllApis(page)
    await page.goto('/')

    const firstCard = page.locator('.v-card').first()
    await firstCard.locator('.mdi-delete').click()
    await page.getByRole('button', { name: 'Cancel' }).click()

    await expect(page.getByText('Delete Batch')).not.toBeVisible()
  })

  test('Upload Documents button navigates to upload page', async ({ page }) => {
    await mockAllApis(page)
    await page.goto('/')

    // Use the header "Upload Documents" button (with the upload icon)
    await page.locator('.v-btn', { hasText: 'Upload Documents' }).first().click()
    await expect(page).toHaveURL('/upload')
  })
})
