<template>
  <v-card variant="outlined" class="mb-3">
    <v-card-title class="d-flex align-center">
      <v-icon :color="typeColor" class="mr-2">{{ typeIcon }}</v-icon>
      <span>{{ documentTypeName }}</span>
      <v-spacer />
      <v-chip :color="confidenceColor" size="small" variant="tonal">
        {{ (extraction.confidence * 100).toFixed(0) }}%
      </v-chip>
    </v-card-title>

    <v-card-subtitle v-if="documentName" class="pb-0">
      {{ documentName }}
    </v-card-subtitle>

    <v-card-text>
      <!-- Key-value pairs from structured_data -->
      <v-table v-if="dataEntries.length > 0" density="compact">
        <tbody>
          <tr v-for="[key, val] in dataEntries" :key="key">
            <td class="text-medium-emphasis" style="width: 40%">{{ formatKey(key) }}</td>
            <td>{{ formatValue(val) }}</td>
          </tr>
        </tbody>
      </v-table>

      <div v-else class="text-medium-emphasis text-body-2 py-2">
        No structured data extracted
      </div>

      <!-- Metadata footer -->
      <div class="d-flex mt-3 text-caption text-medium-emphasis">
        <span v-if="extraction.model_used" class="mr-3">
          <v-icon size="x-small" class="mr-1">mdi-robot</v-icon>
          {{ extraction.model_used }}
        </span>
        <span v-if="extraction.processing_time_ms > 0">
          <v-icon size="x-small" class="mr-1">mdi-timer-outline</v-icon>
          {{ (extraction.processing_time_ms / 1000).toFixed(1) }}s
        </span>
      </div>
    </v-card-text>

    <v-card-actions v-if="showActions">
      <v-btn size="small" variant="text" @click="$emit('view-details', extraction)">
        View Details
      </v-btn>
      <v-btn
        v-if="extraction.raw_text"
        size="small"
        variant="text"
        @click="showRawText = !showRawText"
      >
        {{ showRawText ? 'Hide' : 'Show' }} Raw Text
      </v-btn>
    </v-card-actions>

    <v-expand-transition>
      <v-card-text v-if="showRawText && extraction.raw_text">
        <v-textarea
          :model-value="extraction.raw_text"
          variant="outlined"
          readonly
          rows="6"
          density="compact"
          label="Raw Extracted Text"
        />
      </v-card-text>
    </v-expand-transition>
  </v-card>
</template>

<script setup lang="ts">
import { computed, ref } from 'vue'
import type { Extraction } from '../api/client'

const props = withDefaults(
  defineProps<{
    extraction: Extraction
    documentName?: string
    showActions?: boolean
  }>(),
  { showActions: true },
)

defineEmits<{
  (e: 'view-details', extraction: Extraction): void
}>()

const showRawText = ref(false)

const skipKeys = ['confidence', 'document_type', 'raw_response', 'parse_error']

const dataEntries = computed(() => {
  if (!props.extraction.structured_data) return []
  return Object.entries(props.extraction.structured_data).filter(
    ([key]) => !skipKeys.includes(key),
  )
})

const documentTypeName = computed(() => {
  return props.extraction.document_type
    .split('_')
    .map((w) => w.charAt(0).toUpperCase() + w.slice(1))
    .join(' ')
})

const confidenceColor = computed(() => {
  if (props.extraction.confidence >= 0.8) return 'success'
  if (props.extraction.confidence >= 0.5) return 'warning'
  return 'error'
})

const typeColor = computed(() => {
  switch (props.extraction.document_type) {
    case 'invoice': return 'blue'
    case 'bank_statement': return 'green'
    case 'payment': return 'orange'
    case 'receipt': return 'purple'
    default: return 'grey'
  }
})

const typeIcon = computed(() => {
  switch (props.extraction.document_type) {
    case 'invoice': return 'mdi-receipt-text'
    case 'bank_statement': return 'mdi-bank'
    case 'payment': return 'mdi-credit-card'
    case 'receipt': return 'mdi-receipt'
    default: return 'mdi-file-document'
  }
})

function formatKey(key: string): string {
  return key
    .replace(/_/g, ' ')
    .replace(/\b\w/g, (c) => c.toUpperCase())
}

function formatValue(val: unknown): string {
  if (val === null || val === undefined) return '-'
  if (typeof val === 'object') return JSON.stringify(val)
  return String(val)
}
</script>
