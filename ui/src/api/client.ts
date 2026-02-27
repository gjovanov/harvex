const BASE = ''

async function request<T>(path: string, options?: RequestInit): Promise<T> {
  const res = await fetch(`${BASE}${path}`, {
    headers: { 'Content-Type': 'application/json', ...options?.headers },
    ...options,
  })
  if (!res.ok) {
    const body = await res.json().catch(() => ({}))
    throw new Error(body.error || `HTTP ${res.status}`)
  }
  return res.json()
}

export const api = {
  health: () => request('/health'),

  // Batches
  listBatches: () => request('/api/batch'),
  createBatch: (name: string, modelName?: string) =>
    request('/api/batch', {
      method: 'POST',
      body: JSON.stringify({ name, model_name: modelName }),
    }),
  getBatch: (id: string) => request(`/api/batch/${id}`),

  // Documents
  listDocuments: (batchId: string) => request(`/api/document?batch_id=${batchId}`),
  getDocument: (id: string) => request(`/api/document/${id}`),
  deleteDocument: (id: string) => request(`/api/document/${id}`, { method: 'DELETE' }),

  // Upload (multipart)
  uploadFiles: (files: File[], batchName?: string, modelName?: string) => {
    const form = new FormData()
    files.forEach((f) => form.append('files[]', f))
    if (batchName) form.append('batch_name', batchName)
    if (modelName) form.append('model_name', modelName)
    return fetch('/api/document/upload', { method: 'POST', body: form }).then((r) => r.json())
  },

  // Extractions
  listExtractions: (batchId: string) => request(`/api/batch/${batchId}/extraction`),
  getExtraction: (batchId: string, extractionId: string) =>
    request(`/api/batch/${batchId}/extraction/${extractionId}`),

  // Export
  exportJsonUrl: (batchId: string) => `/api/export/json/${batchId}`,
  exportExcelUrl: (batchId: string) => `/api/export/excel/${batchId}`,
}
