import { ref, computed } from 'vue'
import { defineStore } from 'pinia'
import { api, type Batch, type ProgressEvent } from '../api/client'

export const useBatchesStore = defineStore('batches', () => {
  const batches = ref<Batch[]>([])
  const current = ref<Batch | null>(null)
  const loading = ref(false)
  const error = ref<string | null>(null)
  const processing = ref<Map<string, ProgressEvent>>(new Map())

  const sortedBatches = computed(() =>
    [...batches.value].sort((a, b) => b.created_at.localeCompare(a.created_at)),
  )

  async function fetchAll() {
    loading.value = true
    error.value = null
    try {
      batches.value = await api.listBatches()
    } catch (e) {
      error.value = (e as Error).message
    } finally {
      loading.value = false
    }
  }

  async function fetchOne(id: string) {
    loading.value = true
    error.value = null
    try {
      current.value = await api.getBatch(id)
    } catch (e) {
      error.value = (e as Error).message
    } finally {
      loading.value = false
    }
  }

  async function create(name: string, modelName?: string) {
    error.value = null
    try {
      const batch = await api.createBatch(name, modelName)
      batches.value.unshift(batch)
      return batch
    } catch (e) {
      error.value = (e as Error).message
      throw e
    }
  }

  async function deleteBatch(id: string) {
    error.value = null
    try {
      await api.deleteBatch(id)
      batches.value = batches.value.filter((b) => b.id !== id)
      if (current.value?.id === id) current.value = null
    } catch (e) {
      error.value = (e as Error).message
      throw e
    }
  }

  async function process(id: string) {
    error.value = null
    try {
      await api.processBatch(id)
      // Update local status
      const batch = batches.value.find((b) => b.id === id)
      if (batch) batch.status = 'processing'
      if (current.value?.id === id) current.value.status = 'processing'
    } catch (e) {
      error.value = (e as Error).message
      throw e
    }
  }

  function startProgressStream(batchId: string): EventSource {
    const es = api.progressStream(batchId)

    es.onmessage = (event) => {
      try {
        const data: ProgressEvent = JSON.parse(event.data)
        processing.value.set(batchId, data)

        // Update batch in list
        const batch = batches.value.find((b) => b.id === batchId)
        if (batch) {
          batch.processed_files = data.processed
          batch.failed_files = data.failed
          batch.total_files = data.total
          if (data.status === 'completed' || data.status === 'failed' || data.status === 'partially_completed') {
            batch.status = data.status
          }
        }

        // Update current
        if (current.value?.id === batchId) {
          current.value.processed_files = data.processed
          current.value.failed_files = data.failed
          current.value.total_files = data.total
          if (data.status === 'completed' || data.status === 'failed' || data.status === 'partially_completed') {
            current.value.status = data.status
            es.close()
            processing.value.delete(batchId)
          }
        }
      } catch {
        // ignore parse errors
      }
    }

    es.onerror = () => {
      es.close()
      processing.value.delete(batchId)
    }

    return es
  }

  function getProgress(batchId: string): ProgressEvent | undefined {
    return processing.value.get(batchId)
  }

  return {
    batches,
    current,
    loading,
    error,
    sortedBatches,
    processing,
    fetchAll,
    fetchOne,
    create,
    deleteBatch,
    process,
    startProgressStream,
    getProgress,
  }
})
