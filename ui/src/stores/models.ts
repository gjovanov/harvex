import { ref } from 'vue'
import { defineStore } from 'pinia'
import { api, type ModelInfo, type ModelHealth } from '../api/client'

export const useModelsStore = defineStore('models', () => {
  const info = ref<ModelInfo | null>(null)
  const health = ref<ModelHealth | null>(null)
  const available = ref<Array<Record<string, unknown>>>([])
  const loading = ref(false)
  const error = ref<string | null>(null)

  async function fetchInfo() {
    loading.value = true
    error.value = null
    try {
      info.value = await api.getModelInfo()
    } catch (e) {
      error.value = (e as Error).message
    } finally {
      loading.value = false
    }
  }

  async function checkHealth() {
    error.value = null
    try {
      health.value = await api.checkModelHealth()
    } catch (e) {
      error.value = (e as Error).message
    }
  }

  async function fetchAvailable() {
    error.value = null
    try {
      const result = await api.listModels()
      available.value = result.available
    } catch (e) {
      error.value = (e as Error).message
    }
  }

  async function switchModel(modelName: string) {
    error.value = null
    try {
      const result = await api.switchModel(modelName)
      if (info.value) {
        info.value.model_name = result.current_model
      }
      return result
    } catch (e) {
      error.value = (e as Error).message
      throw e
    }
  }

  async function updateSettings(settings: {
    api_url?: string
    api_key?: string
    temperature?: number
    max_tokens?: number
    context_size?: number
  }) {
    error.value = null
    try {
      const result = await api.updateModelSettings(settings)
      info.value = result.current
      return result
    } catch (e) {
      error.value = (e as Error).message
      throw e
    }
  }

  return {
    info,
    health,
    available,
    loading,
    error,
    fetchInfo,
    checkHealth,
    fetchAvailable,
    switchModel,
    updateSettings,
  }
})
