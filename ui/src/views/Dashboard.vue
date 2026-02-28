<template>
  <div>
    <div class="d-flex align-center mb-4">
      <h1 class="text-h5">Batches</h1>
      <v-spacer />
      <v-btn color="primary" prepend-icon="mdi-upload" to="/upload">
        Upload Documents
      </v-btn>
    </div>

    <v-alert v-if="batchesStore.error" type="error" closable class="mb-4" @click:close="batchesStore.error = null">
      {{ batchesStore.error }}
    </v-alert>

    <v-progress-linear v-if="batchesStore.loading" indeterminate color="primary" class="mb-4" />

    <div v-if="batchesStore.sortedBatches.length === 0 && !batchesStore.loading" class="text-center py-12">
      <v-icon size="80" color="grey-lighten-1" class="mb-4">mdi-folder-open-outline</v-icon>
      <div class="text-h6 text-medium-emphasis mb-2">No batches yet</div>
      <div class="text-body-2 text-medium-emphasis mb-4">
        Upload document batches to get started with extraction.
      </div>
      <v-btn color="primary" to="/upload">Upload Documents</v-btn>
    </div>

    <v-row v-else>
      <v-col v-for="batch in batchesStore.sortedBatches" :key="batch.id" cols="12" sm="6" lg="4">
        <v-card
          :to="`/batch/${batch.id}`"
          hover
          class="h-100"
        >
          <v-card-title class="d-flex align-center">
            <v-icon :color="statusColor(batch.status)" class="mr-2" size="small">
              {{ statusIcon(batch.status) }}
            </v-icon>
            {{ batch.name }}
          </v-card-title>

          <v-card-text>
            <div class="d-flex align-center mb-2">
              <v-chip :color="statusColor(batch.status)" size="small" variant="tonal">
                {{ batch.status }}
              </v-chip>
              <v-spacer />
              <span class="text-caption text-medium-emphasis">
                {{ batch.total_files }} file{{ batch.total_files !== 1 ? 's' : '' }}
              </span>
            </div>

            <v-progress-linear
              v-if="batch.status === 'processing' || batch.total_files > 0"
              :model-value="progressPercent(batch)"
              :color="statusColor(batch.status)"
              height="4"
              rounded
              class="mb-2"
            />

            <div class="d-flex text-caption text-medium-emphasis">
              <span v-if="batch.processed_files > 0" class="text-success mr-2">
                {{ batch.processed_files }} done
              </span>
              <span v-if="batch.failed_files > 0" class="text-error mr-2">
                {{ batch.failed_files }} failed
              </span>
              <v-spacer />
              <span>{{ formatDate(batch.created_at) }}</span>
            </div>

            <div v-if="batch.model_name" class="text-caption text-medium-emphasis mt-1">
              <v-icon size="x-small" class="mr-1">mdi-robot</v-icon>
              {{ batch.model_name }}
            </div>
          </v-card-text>

          <v-card-actions @click.prevent>
            <v-btn
              v-if="batch.status === 'pending'"
              size="small"
              color="primary"
              variant="text"
              @click.prevent="startProcessing(batch.id)"
            >
              Process
            </v-btn>
            <v-spacer />
            <v-btn
              size="small"
              color="error"
              variant="text"
              icon="mdi-delete"
              @click.prevent="confirmDelete(batch)"
            />
          </v-card-actions>
        </v-card>
      </v-col>
    </v-row>

    <!-- Delete confirmation dialog -->
    <v-dialog v-model="deleteDialog" max-width="400">
      <v-card>
        <v-card-title>Delete Batch</v-card-title>
        <v-card-text>
          Are you sure you want to delete "{{ batchToDelete?.name }}"?
          This will remove all documents, extractions, and files.
        </v-card-text>
        <v-card-actions>
          <v-spacer />
          <v-btn variant="text" @click="deleteDialog = false">Cancel</v-btn>
          <v-btn color="error" variant="flat" :loading="deleting" @click="doDelete">Delete</v-btn>
        </v-card-actions>
      </v-card>
    </v-dialog>
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted } from 'vue'
import { useBatchesStore } from '../stores/batches'
import { useSnackbar } from '../composables/useSnackbar'
import type { Batch } from '../api/client'

const batchesStore = useBatchesStore()
const { showSuccess, showError } = useSnackbar()

const deleteDialog = ref(false)
const batchToDelete = ref<Batch | null>(null)
const deleting = ref(false)

onMounted(() => {
  batchesStore.fetchAll()
})

function statusColor(status: string): string {
  switch (status) {
    case 'completed': return 'success'
    case 'processing': return 'primary'
    case 'partially_completed': return 'warning'
    case 'failed': return 'error'
    default: return 'grey'
  }
}

function statusIcon(status: string): string {
  switch (status) {
    case 'completed': return 'mdi-check-circle'
    case 'processing': return 'mdi-progress-clock'
    case 'partially_completed': return 'mdi-alert-circle'
    case 'failed': return 'mdi-close-circle'
    default: return 'mdi-clock-outline'
  }
}

function progressPercent(batch: Batch): number {
  if (batch.total_files === 0) return 0
  return ((batch.processed_files + batch.failed_files) / batch.total_files) * 100
}

function formatDate(dateStr: string): string {
  try {
    return new Date(dateStr).toLocaleDateString(undefined, {
      month: 'short',
      day: 'numeric',
      hour: '2-digit',
      minute: '2-digit',
    })
  } catch {
    return dateStr
  }
}

async function startProcessing(id: string) {
  try {
    await batchesStore.process(id)
    batchesStore.startProgressStream(id)
    showSuccess('Processing started')
  } catch (e) {
    showError((e as Error).message)
  }
}

function confirmDelete(batch: Batch) {
  batchToDelete.value = batch
  deleteDialog.value = true
}

async function doDelete() {
  if (!batchToDelete.value) return
  deleting.value = true
  try {
    await batchesStore.deleteBatch(batchToDelete.value.id)
    showSuccess('Batch deleted')
    deleteDialog.value = false
  } catch (e) {
    showError((e as Error).message)
  } finally {
    deleting.value = false
  }
}
</script>
