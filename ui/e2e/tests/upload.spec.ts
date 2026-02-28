import { test, expect } from '@playwright/test'
import { mockAllApis } from '../fixtures/api-mock'

test.describe('Upload', () => {
  test('renders upload page with drop zone and settings', async ({ page }) => {
    await mockAllApis(page)
    await page.goto('/upload')

    await expect(page.locator('h1', { hasText: 'Upload Documents' })).toBeVisible()
    await expect(page.locator('.v-card-title', { hasText: 'Batch Settings' })).toBeVisible()
    await expect(page.getByLabel('Batch Name')).toBeVisible()
    await expect(page.getByRole('button', { name: /Upload & Process/ })).toBeVisible()
  })

  test('upload button is disabled when no files selected', async ({ page }) => {
    await mockAllApis(page)
    await page.goto('/upload')

    const uploadBtn = page.getByRole('button', { name: /Upload & Process/ })
    await expect(uploadBtn).toBeDisabled()
  })

  test('shows file count after selecting files', async ({ page }) => {
    await mockAllApis(page)
    await page.goto('/upload')

    // Simulate file input via the hidden input
    const fileInput = page.locator('input[type="file"]')
    await fileInput.setInputFiles([
      { name: 'test.pdf', mimeType: 'application/pdf', buffer: Buffer.from('fake pdf') },
      { name: 'test.png', mimeType: 'image/png', buffer: Buffer.from('fake png') },
    ])

    await expect(page.getByText('2 files selected')).toBeVisible()
  })

  test('auto-generates batch name on file selection', async ({ page }) => {
    await mockAllApis(page)
    await page.goto('/upload')

    const fileInput = page.locator('input[type="file"]')
    await fileInput.setInputFiles([
      { name: 'test.pdf', mimeType: 'application/pdf', buffer: Buffer.from('fake pdf') },
    ])

    const batchNameInput = page.getByLabel('Batch Name')
    await expect(batchNameInput).not.toHaveValue('')
    // Should contain "Batch" prefix
    const val = await batchNameInput.inputValue()
    expect(val).toContain('Batch')
  })

  test('back button navigates to dashboard', async ({ page }) => {
    await mockAllApis(page)
    await page.goto('/upload')

    await page.locator('.v-btn').filter({ has: page.locator('.mdi-arrow-left') }).first().click()
    await expect(page).toHaveURL('/')
  })

  test('upload sends multipart request with files', async ({ page }) => {
    await mockAllApis(page)

    // Override upload route AFTER mockAllApis (last route wins in Playwright)
    let uploadCalled = false
    await page.route('**/api/document/upload', (route) => {
      uploadCalled = true
      return route.fulfill({
        status: 200,
        contentType: 'application/json',
        body: JSON.stringify({
          batch: {
            id: 'batch-new', name: 'Upload Batch', status: 'pending',
            total_files: 1, processed_files: 0, failed_files: 0,
            model_name: null, created_at: '2025-01-15T10:00:00',
            updated_at: '2025-01-15T10:00:00', completed_at: null,
          },
          documents: [{
            id: 'doc-new', batch_id: 'batch-new', filename: 'abc.pdf',
            original_name: 'invoice.pdf', content_type: 'application/pdf',
            file_size: 8, file_path: '/tmp/uploads/batch-new/abc.pdf',
            status: 'pending', error_message: null,
            created_at: '2025-01-15T10:00:00', updated_at: '2025-01-15T10:00:00',
          }],
        }),
      })
    })

    await page.goto('/upload')

    const fileInput = page.locator('input[type="file"]')
    await fileInput.setInputFiles([
      { name: 'invoice.pdf', mimeType: 'application/pdf', buffer: Buffer.from('fake pdf') },
    ])

    const uploadPromise = page.waitForRequest('**/api/document/upload')
    await page.getByRole('button', { name: /Upload & Process/ }).click()

    const request = await uploadPromise
    expect(request.method()).toBe('POST')
    expect(uploadCalled).toBe(true)

    // After upload, a success snackbar should appear
    await expect(page.getByText(/Uploaded \d+ file/)).toBeVisible({ timeout: 5000 })
  })
})
