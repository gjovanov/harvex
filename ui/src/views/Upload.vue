<template>
  <div>
    <div class="d-flex align-center mb-4">
      <v-btn icon variant="text" to="/" class="mr-2">
        <v-icon>mdi-arrow-left</v-icon>
      </v-btn>
      <h1 class="text-h5">Upload Documents</h1>
    </div>

    <v-row>
      <v-col cols="12" md="8">
        <FileDropZone ref="dropZone" @files-selected="onFilesSelected" />
      </v-col>

      <v-col cols="12" md="4">
        <v-card>
          <v-card-title>Batch Settings</v-card-title>
          <v-card-text>
            <v-text-field
              v-model="batchName"
              label="Batch Name"
              variant="outlined"
              density="compact"
              placeholder="e.g. January Invoices"
              class="mb-3"
            />

            <ModelSelector @model-changed="onModelChanged" />

            <div class="text-caption text-medium-emphasis mt-3">
              {{ files.length }} file{{ files.length !== 1 ? 's' : '' }} selected
              <span v-if="totalSize > 0">({{ formatSize(totalSize) }} total)</span>
            </div>
          </v-card-text>

          <v-card-actions>
            <v-btn
              color="primary"
              variant="flat"
              block
              :disabled="files.length === 0 || uploading"
              :loading="uploading"
              prepend-icon="mdi-upload"
              @click="doUpload"
            >
              Upload & Process
            </v-btn>
          </v-card-actions>

          <v-card-text v-if="uploading">
            <v-progress-linear indeterminate color="primary" class="mb-2" />
            <div class="text-caption text-medium-emphasis text-center">
              Uploading {{ files.length }} files...
            </div>
          </v-card-text>
        </v-card>
      </v-col>
    </v-row>
  </div>
</template>

<script setup lang="ts">
import { ref, computed } from 'vue'
import { useRouter } from 'vue-router'
import { useDocumentsStore } from '../stores/documents'
import { useBatchesStore } from '../stores/batches'
import { useSnackbar } from '../composables/useSnackbar'
import FileDropZone from '../components/FileDropZone.vue'
import ModelSelector from '../components/ModelSelector.vue'

const router = useRouter()
const documentsStore = useDocumentsStore()
const batchesStore = useBatchesStore()
const { showSuccess, showError } = useSnackbar()

const dropZone = ref<InstanceType<typeof FileDropZone>>()
const files = ref<File[]>([])
const batchName = ref('')
const modelName = ref<string | undefined>()
const uploading = ref(false)

const totalSize = computed(() => files.value.reduce((sum, f) => sum + f.size, 0))

function onFilesSelected(selectedFiles: File[]) {
  files.value = selectedFiles
  if (!batchName.value && selectedFiles.length > 0) {
    const date = new Date().toLocaleDateString(undefined, { month: 'short', day: 'numeric', year: 'numeric' })
    batchName.value = `Batch ${date}`
  }
}

function onModelChanged(name: string) {
  modelName.value = name
}

async function doUpload() {
  if (files.value.length === 0) return
  uploading.value = true
  try {
    const result = await documentsStore.upload(
      files.value,
      batchName.value || undefined,
      modelName.value,
    )

    showSuccess(`Uploaded ${result.documents.length} files`)

    try {
      await batchesStore.process(result.batch.id)
      batchesStore.startProgressStream(result.batch.id)
    } catch {
      // Processing start failed, but upload succeeded
    }

    router.push(`/batch/${result.batch.id}`)
  } catch (e) {
    showError((e as Error).message)
  } finally {
    uploading.value = false
  }
}

function formatSize(bytes: number): string {
  if (bytes < 1024) return `${bytes} B`
  if (bytes < 1024 * 1024) return `${(bytes / 1024).toFixed(1)} KB`
  return `${(bytes / (1024 * 1024)).toFixed(1)} MB`
}
</script>
