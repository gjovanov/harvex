<template>
  <v-select
    v-model="selectedModel"
    :items="modelItems"
    :loading="loading"
    label="LLM Model"
    variant="outlined"
    density="compact"
    hide-details
    @update:model-value="onSelect"
  >
    <template #prepend-inner>
      <v-icon size="small">mdi-robot</v-icon>
    </template>
    <template #append>
      <v-btn
        icon
        size="x-small"
        variant="text"
        :loading="loading"
        @click.stop="refresh"
      >
        <v-icon size="small">mdi-refresh</v-icon>
      </v-btn>
    </template>
  </v-select>
</template>

<script setup lang="ts">
import { ref, onMounted, watch } from 'vue'
import { useModelsStore } from '../stores/models'

const emit = defineEmits<{
  (e: 'model-changed', modelName: string): void
}>()

const modelsStore = useModelsStore()
const selectedModel = ref('')
const loading = ref(false)

const modelItems = ref<string[]>([])

async function refresh() {
  loading.value = true
  try {
    await Promise.all([modelsStore.fetchInfo(), modelsStore.fetchAvailable()])
    if (modelsStore.info) {
      selectedModel.value = modelsStore.info.model_name
    }
    modelItems.value = modelsStore.available
      .map((m) => (m.id || m.name || m.model || '') as string)
      .filter(Boolean)
    // Ensure current model is in the list
    if (selectedModel.value && !modelItems.value.includes(selectedModel.value)) {
      modelItems.value.unshift(selectedModel.value)
    }
  } finally {
    loading.value = false
  }
}

async function onSelect(modelName: string) {
  if (!modelName) return
  try {
    await modelsStore.switchModel(modelName)
    emit('model-changed', modelName)
  } catch {
    // revert
    if (modelsStore.info) {
      selectedModel.value = modelsStore.info.model_name
    }
  }
}

watch(
  () => modelsStore.info?.model_name,
  (name) => {
    if (name) selectedModel.value = name
  },
)

onMounted(refresh)
</script>
