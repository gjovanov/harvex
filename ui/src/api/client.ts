const BASE = ''

async function request<T>(path: string, options?: RequestInit): Promise<T> {
  const res = await fetch(`${BASE}${path}`, {
    headers: { 'Content-Type': 'application/json', ...options?.headers },
    ...options,
  })
  if (!res.ok) {
    const body = await res.json().catch(() => ({}))
    throw new Error(body.error || body.message || `HTTP ${res.status}`)
  }
  return res.json()
}

export interface Batch {
  id: string
  name: string
  status: string
  total_files: number
  processed_files: number
  failed_files: number
  model_name: string | null
  created_at: string
  updated_at: string
  completed_at: string | null
}

export interface Document {
  id: string
  batch_id: string
  filename: string
  original_name: string
  content_type: string
  file_size: number
  file_path: string
  status: string
  error_message: string | null
  created_at: string
  updated_at: string
}

export interface Extraction {
  id: string
  document_id: string
  batch_id: string
  document_type: string
  raw_text: string | null
  structured_data: Record<string, unknown> | null
  confidence: number
  model_used: string | null
  processing_time_ms: number
  created_at: string
}

export interface ProgressEvent {
  batch_id: string
  document_id: string
  document_name: string
  status: string
  message: string
  processed: number
  failed: number
  total: number
}

export interface ModelInfo {
  model_name: string
  api_url: string
  context_size: number
  temperature: number
  max_tokens: number
}

export interface ModelHealth {
  model_name: string
  api_url: string
  reachable: boolean
}

export const api = {
  health: () => request('/health'),

  // Batches
  listBatches: () => request<Batch[]>('/api/batch'),
  createBatch: (name: string, modelName?: string) =>
    request<Batch>('/api/batch', {
      method: 'POST',
      body: JSON.stringify({ name, model_name: modelName }),
    }),
  getBatch: (id: string) => request<Batch>(`/api/batch/${id}`),
  deleteBatch: (id: string) =>
    request<{ message: string; batch_id: string; files_removed: number }>(`/api/batch/${id}`, {
      method: 'DELETE',
    }),
  processBatch: (id: string) =>
    request<{ status: string; batch_id: string; message: string }>(`/api/batch/${id}/process`, {
      method: 'POST',
    }),

  // Documents
  listDocuments: (batchId: string) => request<Document[]>(`/api/document?batch_id=${batchId}`),
  getDocument: (id: string) => request<Document>(`/api/document/${id}`),
  deleteDocument: (id: string) =>
    request<{ deleted: boolean; id: string }>(`/api/document/${id}`, { method: 'DELETE' }),

  // Upload (multipart — no Content-Type header, browser sets boundary)
  uploadFiles: async (files: File[], batchName?: string, modelName?: string) => {
    const form = new FormData()
    files.forEach((f) => form.append('files[]', f))
    if (batchName) form.append('batch_name', batchName)
    if (modelName) form.append('model_name', modelName)
    const res = await fetch('/api/document/upload', { method: 'POST', body: form })
    if (!res.ok) {
      const body = await res.json().catch(() => ({}))
      throw new Error(body.error || body.message || `Upload failed: HTTP ${res.status}`)
    }
    return res.json() as Promise<{ batch: Batch; documents: Document[] }>
  },

  // Extractions
  listExtractions: (batchId: string) =>
    request<Extraction[]>(`/api/batch/${batchId}/extraction`),
  getExtraction: (batchId: string, extractionId: string) =>
    request<Extraction>(`/api/batch/${batchId}/extraction/${extractionId}`),

  // Export URLs (for download links)
  exportJsonUrl: (batchId: string, filter?: { document_type?: string; min_confidence?: number }) => {
    const params = new URLSearchParams()
    if (filter?.document_type) params.set('document_type', filter.document_type)
    if (filter?.min_confidence != null) params.set('min_confidence', String(filter.min_confidence))
    const qs = params.toString()
    return `/api/export/json/${batchId}${qs ? `?${qs}` : ''}`
  },
  exportExcelUrl: (batchId: string, filter?: { document_type?: string; min_confidence?: number }) => {
    const params = new URLSearchParams()
    if (filter?.document_type) params.set('document_type', filter.document_type)
    if (filter?.min_confidence != null) params.set('min_confidence', String(filter.min_confidence))
    const qs = params.toString()
    return `/api/export/excel/${batchId}${qs ? `?${qs}` : ''}`
  },
  exportCsvUrl: (batchId: string, filter?: { document_type?: string; min_confidence?: number }) => {
    const params = new URLSearchParams()
    if (filter?.document_type) params.set('document_type', filter.document_type)
    if (filter?.min_confidence != null) params.set('min_confidence', String(filter.min_confidence))
    const qs = params.toString()
    return `/api/export/csv/${batchId}${qs ? `?${qs}` : ''}`
  },

  // SSE progress stream
  progressStream: (batchId: string): EventSource => {
    return new EventSource(`/api/batch/${batchId}/progress`)
  },

  // Model management
  getModelInfo: () => request<ModelInfo>('/api/model'),
  switchModel: (modelName: string) =>
    request<{ previous_model: string; current_model: string; message: string }>('/api/model/switch', {
      method: 'POST',
      body: JSON.stringify({ model_name: modelName }),
    }),
  updateModelSettings: (settings: {
    api_url?: string
    api_key?: string
    temperature?: number
    max_tokens?: number
    context_size?: number
  }) =>
    request<{ message: string; current: ModelInfo }>('/api/model/settings', {
      method: 'POST',
      body: JSON.stringify(settings),
    }),
  checkModelHealth: () => request<ModelHealth>('/api/model/health'),
  listModels: () =>
    request<{ current_model: string; available: Array<Record<string, unknown>> }>('/api/model/list'),
}
