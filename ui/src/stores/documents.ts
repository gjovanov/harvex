import { ref } from 'vue'
import { defineStore } from 'pinia'
import { api, type Document } from '../api/client'

export const useDocumentsStore = defineStore('documents', () => {
  const documents = ref<Document[]>([])
  const current = ref<Document | null>(null)
  const loading = ref(false)
  const error = ref<string | null>(null)

  async function fetchByBatch(batchId: string) {
    loading.value = true
    error.value = null
    try {
      documents.value = await api.listDocuments(batchId)
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
      current.value = await api.getDocument(id)
    } catch (e) {
      error.value = (e as Error).message
    } finally {
      loading.value = false
    }
  }

  async function deleteDocument(id: string) {
    error.value = null
    try {
      await api.deleteDocument(id)
      documents.value = documents.value.filter((d) => d.id !== id)
      if (current.value?.id === id) current.value = null
    } catch (e) {
      error.value = (e as Error).message
      throw e
    }
  }

  async function upload(files: File[], batchName?: string, modelName?: string) {
    loading.value = true
    error.value = null
    try {
      const result = await api.uploadFiles(files, batchName, modelName)
      return result
    } catch (e) {
      error.value = (e as Error).message
      throw e
    } finally {
      loading.value = false
    }
  }

  function formatFileSize(bytes: number): string {
    if (bytes < 1024) return `${bytes} B`
    if (bytes < 1024 * 1024) return `${(bytes / 1024).toFixed(1)} KB`
    return `${(bytes / (1024 * 1024)).toFixed(1)} MB`
  }

  return {
    documents,
    current,
    loading,
    error,
    fetchByBatch,
    fetchOne,
    deleteDocument,
    upload,
    formatFileSize,
  }
})
