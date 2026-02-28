import { type Page } from '@playwright/test'
import {
  sampleBatches,
  sampleDocuments,
  sampleExtractions,
  makeModelInfo,
  makeModelHealth,
  makeBatch,
} from './mock-data'

/**
 * Set up route interception to mock all API endpoints.
 * Tests run without the Rust backend — all responses are mocked.
 */
export async function mockAllApis(page: Page, overrides: MockOverrides = {}) {
  const batches = overrides.batches ?? sampleBatches
  const documents = overrides.documents ?? sampleDocuments
  const extractions = overrides.extractions ?? sampleExtractions
  const modelInfo = overrides.modelInfo ?? makeModelInfo()
  const modelHealth = overrides.modelHealth ?? makeModelHealth()

  // Health
  await page.route('**/health', (route) =>
    route.fulfill({ status: 200, contentType: 'application/json', body: JSON.stringify({ status: 'ok' }) }),
  )

  // Batches
  await page.route('**/api/batch', (route) => {
    if (route.request().method() === 'POST') {
      const body = route.request().postDataJSON()
      const newBatch = makeBatch({
        id: 'batch-new',
        name: body.name || 'New Batch',
        model_name: body.model_name || null,
        status: 'pending',
        total_files: 0,
        processed_files: 0,
      })
      return route.fulfill({ status: 200, contentType: 'application/json', body: JSON.stringify(newBatch) })
    }
    return route.fulfill({ status: 200, contentType: 'application/json', body: JSON.stringify(batches) })
  })

  await page.route('**/api/batch/*/process', (route) =>
    route.fulfill({
      status: 200,
      contentType: 'application/json',
      body: JSON.stringify({ status: 'processing', batch_id: 'batch-002', message: 'Processing started' }),
    }),
  )

  await page.route('**/api/batch/*/progress', (route) =>
    route.fulfill({
      status: 200,
      contentType: 'text/event-stream',
      body: 'data: {"batch_id":"batch-002","status":"completed","processed":3,"failed":0,"total":3,"message":"done"}\n\n',
    }),
  )

  await page.route(/\/api\/batch\/[^/]+\/extraction\/[^/]+$/, (route) => {
    const url = route.request().url()
    const eid = url.split('/').pop()
    const ext = extractions.find((e) => e.id === eid) || extractions[0]
    return route.fulfill({ status: 200, contentType: 'application/json', body: JSON.stringify(ext) })
  })

  await page.route(/\/api\/batch\/[^/]+\/extraction$/, (route) =>
    route.fulfill({ status: 200, contentType: 'application/json', body: JSON.stringify(extractions) }),
  )

  await page.route(/\/api\/batch\/[^/]+$/, (route) => {
    if (route.request().method() === 'DELETE') {
      return route.fulfill({
        status: 200,
        contentType: 'application/json',
        body: JSON.stringify({ message: 'Batch deleted', batch_id: 'batch-001', files_removed: 3 }),
      })
    }
    const url = route.request().url()
    const id = url.split('/').pop()
    const batch = batches.find((b) => b.id === id) || batches[0]
    return route.fulfill({ status: 200, contentType: 'application/json', body: JSON.stringify(batch) })
  })

  // Documents
  await page.route('**/api/document/upload', (route) =>
    route.fulfill({
      status: 200,
      contentType: 'application/json',
      body: JSON.stringify({
        batch: makeBatch({ id: 'batch-new', name: 'Upload Batch', status: 'pending', total_files: 2 }),
        documents: documents.slice(0, 2),
      }),
    }),
  )

  await page.route(/\/api\/document\/[^/]+$/, (route) => {
    if (route.request().method() === 'DELETE') {
      return route.fulfill({
        status: 200,
        contentType: 'application/json',
        body: JSON.stringify({ deleted: true, id: 'doc-001' }),
      })
    }
    return route.fulfill({ status: 200, contentType: 'application/json', body: JSON.stringify(documents[0]) })
  })

  await page.route('**/api/document?*', (route) =>
    route.fulfill({ status: 200, contentType: 'application/json', body: JSON.stringify(documents) }),
  )

  // Export
  await page.route('**/api/export/json/*', (route) =>
    route.fulfill({
      status: 200,
      contentType: 'application/json',
      headers: { 'Content-Disposition': 'attachment; filename="export.json"' },
      body: JSON.stringify(extractions),
    }),
  )

  await page.route('**/api/export/excel/*', (route) =>
    route.fulfill({
      status: 200,
      contentType: 'application/vnd.openxmlformats-officedocument.spreadsheetml.sheet',
      headers: { 'Content-Disposition': 'attachment; filename="export.xlsx"' },
      body: Buffer.from('PK mock excel'),
    }),
  )

  await page.route('**/api/export/csv/*', (route) =>
    route.fulfill({
      status: 200,
      contentType: 'text/csv',
      headers: { 'Content-Disposition': 'attachment; filename="export.csv"' },
      body: 'id,document_type,confidence\next-001,invoice,0.92',
    }),
  )

  // Models
  await page.route('**/api/model/health', (route) =>
    route.fulfill({ status: 200, contentType: 'application/json', body: JSON.stringify(modelHealth) }),
  )

  await page.route('**/api/model/switch', (route) =>
    route.fulfill({
      status: 200,
      contentType: 'application/json',
      body: JSON.stringify({ previous_model: 'qwen2.5-vl:7b', current_model: 'qwen2.5-vl:3b', message: 'Model switched' }),
    }),
  )

  await page.route('**/api/model/settings', (route) =>
    route.fulfill({
      status: 200,
      contentType: 'application/json',
      body: JSON.stringify({ message: 'Settings updated', current: modelInfo }),
    }),
  )

  await page.route('**/api/model/list', (route) =>
    route.fulfill({
      status: 200,
      contentType: 'application/json',
      body: JSON.stringify({
        current_model: modelInfo.model_name,
        available: [
          { id: 'qwen2.5-vl:7b', name: 'qwen2.5-vl:7b', size: 4_500_000_000 },
          { id: 'qwen2.5-vl:3b', name: 'qwen2.5-vl:3b', size: 2_100_000_000 },
        ],
      }),
    }),
  )

  await page.route('**/api/model', (route) =>
    route.fulfill({ status: 200, contentType: 'application/json', body: JSON.stringify(modelInfo) }),
  )
}

export interface MockOverrides {
  batches?: typeof sampleBatches
  documents?: typeof sampleDocuments
  extractions?: typeof sampleExtractions
  modelInfo?: ReturnType<typeof makeModelInfo>
  modelHealth?: ReturnType<typeof makeModelHealth>
}
