<template>
  <v-card v-if="batch" variant="tonal" class="mb-4">
    <v-card-text>
      <div class="d-flex align-center mb-2">
        <v-icon :color="statusColor" class="mr-2">{{ statusIcon }}</v-icon>
        <span class="text-body-1 font-weight-medium">{{ statusLabel }}</span>
        <v-spacer />
        <span class="text-body-2 text-medium-emphasis">
          {{ batch.processed_files + batch.failed_files }} / {{ batch.total_files }}
        </span>
      </div>

      <v-progress-linear
        :model-value="progressPercent"
        :color="statusColor"
        height="8"
        rounded
        :indeterminate="batch.status === 'processing' && progressPercent === 0"
      />

      <div class="d-flex mt-2 text-caption text-medium-emphasis">
        <span v-if="batch.processed_files > 0" class="text-success mr-3">
          {{ batch.processed_files }} processed
        </span>
        <span v-if="batch.failed_files > 0" class="text-error mr-3">
          {{ batch.failed_files }} failed
        </span>
        <v-spacer />
        <span v-if="progress">{{ progress.document_name || progress.message }}</span>
      </div>
    </v-card-text>
  </v-card>
</template>

<script setup lang="ts">
import { computed } from 'vue'
import type { Batch, ProgressEvent } from '../api/client'

const props = defineProps<{
  batch: Batch
  progress?: ProgressEvent
}>()

const progressPercent = computed(() => {
  if (props.batch.total_files === 0) return 0
  return ((props.batch.processed_files + props.batch.failed_files) / props.batch.total_files) * 100
})

const statusColor = computed(() => {
  switch (props.batch.status) {
    case 'completed': return 'success'
    case 'processing': return 'primary'
    case 'partially_completed': return 'warning'
    case 'failed': return 'error'
    default: return 'grey'
  }
})

const statusIcon = computed(() => {
  switch (props.batch.status) {
    case 'completed': return 'mdi-check-circle'
    case 'processing': return 'mdi-progress-clock'
    case 'partially_completed': return 'mdi-alert-circle'
    case 'failed': return 'mdi-close-circle'
    default: return 'mdi-clock-outline'
  }
})

const statusLabel = computed(() => {
  switch (props.batch.status) {
    case 'completed': return 'Completed'
    case 'processing': return 'Processing...'
    case 'partially_completed': return 'Partially Completed'
    case 'failed': return 'Failed'
    default: return 'Pending'
  }
})
</script>
