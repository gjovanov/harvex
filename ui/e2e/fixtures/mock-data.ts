import type { Batch, Document, Extraction, ModelInfo, ModelHealth } from '../../src/api/client'

export function makeBatch(overrides: Partial<Batch> = {}): Batch {
  return {
    id: 'batch-001',
    name: 'Test Batch',
    status: 'completed',
    total_files: 3,
    processed_files: 3,
    failed_files: 0,
    model_name: 'qwen2.5-vl:7b',
    created_at: '2025-01-15T10:00:00',
    updated_at: '2025-01-15T10:05:00',
    completed_at: '2025-01-15T10:05:00',
    ...overrides,
  }
}

export function makeDocument(overrides: Partial<Document> = {}): Document {
  return {
    id: 'doc-001',
    batch_id: 'batch-001',
    filename: 'abc123.pdf',
    original_name: 'invoice-jan.pdf',
    content_type: 'application/pdf',
    file_size: 245000,
    file_path: '/tmp/uploads/batch-001/abc123.pdf',
    status: 'completed',
    error_message: null,
    created_at: '2025-01-15T10:00:00',
    updated_at: '2025-01-15T10:01:00',
    ...overrides,
  }
}

export function makeExtraction(overrides: Partial<Extraction> = {}): Extraction {
  return {
    id: 'ext-001',
    document_id: 'doc-001',
    batch_id: 'batch-001',
    document_type: 'invoice',
    raw_text: 'Invoice #12345\nDate: 2025-01-15\nTotal: $1,500.00',
    structured_data: {
      invoice_number: '12345',
      date: '2025-01-15',
      total: 1500.0,
      vendor: 'Acme Corp',
    },
    confidence: 0.92,
    model_used: 'qwen2.5-vl:7b',
    processing_time_ms: 3200,
    created_at: '2025-01-15T10:01:00',
    ...overrides,
  }
}

export function makeModelInfo(overrides: Partial<ModelInfo> = {}): ModelInfo {
  return {
    model_name: 'qwen2.5-vl:7b',
    api_url: 'http://localhost:11434/v1',
    context_size: 4096,
    temperature: 0.1,
    max_tokens: 2048,
    ...overrides,
  }
}

export function makeModelHealth(overrides: Partial<ModelHealth> = {}): ModelHealth {
  return {
    model_name: 'qwen2.5-vl:7b',
    api_url: 'http://localhost:11434/v1',
    reachable: true,
    ...overrides,
  }
}

export const sampleBatches: Batch[] = [
  makeBatch({ id: 'batch-001', name: 'January Invoices', status: 'completed', total_files: 5, processed_files: 5 }),
  makeBatch({ id: 'batch-002', name: 'Bank Statements Q4', status: 'pending', total_files: 3, processed_files: 0, completed_at: null }),
  makeBatch({ id: 'batch-003', name: 'Payment Receipts', status: 'processing', total_files: 10, processed_files: 4, failed_files: 1, completed_at: null }),
]

export const sampleDocuments: Document[] = [
  makeDocument({ id: 'doc-001', original_name: 'invoice-jan.pdf', content_type: 'application/pdf', file_size: 245000 }),
  makeDocument({ id: 'doc-002', original_name: 'receipt.png', content_type: 'image/png', file_size: 1200000, filename: 'def456.png' }),
  makeDocument({ id: 'doc-003', original_name: 'transactions.xlsx', content_type: 'application/vnd.openxmlformats-officedocument.spreadsheetml.sheet', file_size: 56000, filename: 'ghi789.xlsx' }),
]

export const sampleExtractions: Extraction[] = [
  makeExtraction({ id: 'ext-001', document_id: 'doc-001', document_type: 'invoice', confidence: 0.92 }),
  makeExtraction({
    id: 'ext-002',
    document_id: 'doc-002',
    document_type: 'receipt',
    confidence: 0.85,
    structured_data: { merchant: 'Coffee Shop', amount: 4.5, date: '2025-01-14' },
  }),
  makeExtraction({
    id: 'ext-003',
    document_id: 'doc-003',
    document_type: 'bank_statement',
    confidence: 0.78,
    structured_data: { account: '****1234', period: 'Q4 2024', transactions: 42 },
  }),
]
