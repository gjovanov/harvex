<template>
  <div>
    <div class="d-flex align-center mb-4">
      <v-btn icon variant="text" to="/" class="mr-2">
        <v-icon>mdi-arrow-left</v-icon>
      </v-btn>
      <h1 class="text-h5">Settings</h1>
    </div>

    <v-row>
      <!-- LLM Model Settings -->
      <v-col cols="12" md="6">
        <v-card>
          <v-card-title>
            <v-icon class="mr-2">mdi-robot</v-icon>
            LLM Model
          </v-card-title>
          <v-card-text>
            <v-progress-linear v-if="modelsStore.loading" indeterminate class="mb-4" />

            <ModelSelector class="mb-4" @model-changed="onModelChanged" />

            <v-text-field
              v-model="form.api_url"
              label="API URL"
              variant="outlined"
              density="compact"
              placeholder="http://localhost:11434/v1"
              class="mb-3"
            />

            <v-text-field
              v-model="form.api_key"
              label="API Key"
              variant="outlined"
              density="compact"
              type="password"
              placeholder="(optional)"
              class="mb-3"
            />

            <v-slider
              v-model="form.temperature"
              label="Temperature"
              min="0"
              max="2"
              step="0.1"
              thumb-label
              class="mb-3"
            />

            <v-text-field
              v-model.number="form.max_tokens"
              label="Max Tokens"
              variant="outlined"
              density="compact"
              type="number"
              class="mb-3"
            />

            <v-text-field
              v-model.number="form.context_size"
              label="Context Size"
              variant="outlined"
              density="compact"
              type="number"
              class="mb-3"
            />
          </v-card-text>
          <v-card-actions>
            <v-btn
              color="primary"
              variant="flat"
              :loading="saving"
              @click="saveSettings"
            >
              Save Settings
            </v-btn>
          </v-card-actions>
        </v-card>
      </v-col>

      <!-- Health & Info -->
      <v-col cols="12" md="6">
        <v-card class="mb-4">
          <v-card-title>
            <v-icon class="mr-2">mdi-heart-pulse</v-icon>
            API Health
          </v-card-title>
          <v-card-text>
            <v-btn variant="outlined" :loading="checking" @click="checkHealth" class="mb-3">
              Check Connection
            </v-btn>

            <div v-if="modelsStore.health">
              <v-alert
                :type="modelsStore.health.reachable ? 'success' : 'error'"
                variant="tonal"
                density="compact"
              >
                {{ modelsStore.health.reachable ? 'API is reachable' : 'API is not reachable' }}
              </v-alert>
              <div class="text-caption text-medium-emphasis mt-2">
                <div>Model: {{ modelsStore.health.model_name }}</div>
                <div>URL: {{ modelsStore.health.api_url }}</div>
              </div>
            </div>
          </v-card-text>
        </v-card>

        <v-card>
          <v-card-title>
            <v-icon class="mr-2">mdi-format-list-bulleted</v-icon>
            Available Models
          </v-card-title>
          <v-card-text>
            <v-btn variant="outlined" :loading="loadingModels" @click="loadAvailable" class="mb-3">
              Refresh List
            </v-btn>

            <v-list v-if="modelsStore.available.length > 0" density="compact">
              <v-list-item
                v-for="(model, i) in modelsStore.available"
                :key="i"
              >
                <v-list-item-title>
                  {{ (model.id || model.name || model.model || 'Unknown') }}
                </v-list-item-title>
                <v-list-item-subtitle v-if="model.size">
                  {{ formatModelSize(model.size as number) }}
                </v-list-item-subtitle>
              </v-list-item>
            </v-list>
            <div v-else class="text-medium-emphasis text-body-2">
              No models found. Check your API connection.
            </div>
          </v-card-text>
        </v-card>
      </v-col>
    </v-row>
  </div>
</template>

<script setup lang="ts">
import { ref, reactive, onMounted, watch } from 'vue'
import { useModelsStore } from '../stores/models'
import { useSnackbar } from '../composables/useSnackbar'
import ModelSelector from '../components/ModelSelector.vue'

const modelsStore = useModelsStore()
const { showSuccess, showError } = useSnackbar()

const form = reactive({
  api_url: '',
  api_key: '',
  temperature: 0.1,
  max_tokens: 2048,
  context_size: 4096,
})

const saving = ref(false)
const checking = ref(false)
const loadingModels = ref(false)

onMounted(async () => {
  await modelsStore.fetchInfo()
  if (modelsStore.info) {
    form.api_url = modelsStore.info.api_url
    form.temperature = modelsStore.info.temperature
    form.max_tokens = modelsStore.info.max_tokens
    form.context_size = modelsStore.info.context_size
  }
})

watch(
  () => modelsStore.info,
  (info) => {
    if (info) {
      form.api_url = info.api_url
      form.temperature = info.temperature
      form.max_tokens = info.max_tokens
      form.context_size = info.context_size
    }
  },
)

function onModelChanged(name: string) {
  showSuccess(`Switched to ${name}`)
}

async function saveSettings() {
  saving.value = true
  try {
    await modelsStore.updateSettings({
      api_url: form.api_url || undefined,
      api_key: form.api_key || undefined,
      temperature: form.temperature,
      max_tokens: form.max_tokens,
      context_size: form.context_size,
    })
    showSuccess('Settings saved')
  } catch (e) {
    showError((e as Error).message)
  } finally {
    saving.value = false
  }
}

async function checkHealth() {
  checking.value = true
  try {
    await modelsStore.checkHealth()
  } catch (e) {
    showError((e as Error).message)
  } finally {
    checking.value = false
  }
}

async function loadAvailable() {
  loadingModels.value = true
  try {
    await modelsStore.fetchAvailable()
  } catch (e) {
    showError((e as Error).message)
  } finally {
    loadingModels.value = false
  }
}

function formatModelSize(bytes: number): string {
  if (bytes < 1024 * 1024 * 1024) return `${(bytes / (1024 * 1024)).toFixed(0)} MB`
  return `${(bytes / (1024 * 1024 * 1024)).toFixed(1)} GB`
}
</script>
