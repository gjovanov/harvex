import { ref, computed } from 'vue'
import { defineStore } from 'pinia'
import { api, type Extraction } from '../api/client'

export const useExtractionsStore = defineStore('extractions', () => {
  const extractions = ref<Extraction[]>([])
  const current = ref<Extraction | null>(null)
  const loading = ref(false)
  const error = ref<string | null>(null)

  const documentTypes = computed(() => {
    const types = new Set(extractions.value.map((e) => e.document_type))
    return [...types].sort()
  })

  const averageConfidence = computed(() => {
    if (extractions.value.length === 0) return 0
    const sum = extractions.value.reduce((acc, e) => acc + e.confidence, 0)
    return sum / extractions.value.length
  })

  async function fetchByBatch(batchId: string) {
    loading.value = true
    error.value = null
    try {
      extractions.value = await api.listExtractions(batchId)
    } catch (e) {
      error.value = (e as Error).message
    } finally {
      loading.value = false
    }
  }

  async function fetchOne(batchId: string, extractionId: string) {
    loading.value = true
    error.value = null
    try {
      current.value = await api.getExtraction(batchId, extractionId)
    } catch (e) {
      error.value = (e as Error).message
    } finally {
      loading.value = false
    }
  }

  function confidenceColor(confidence: number): string {
    if (confidence >= 0.8) return 'success'
    if (confidence >= 0.5) return 'warning'
    return 'error'
  }

  return {
    extractions,
    current,
    loading,
    error,
    documentTypes,
    averageConfidence,
    fetchByBatch,
    fetchOne,
    confidenceColor,
  }
})
