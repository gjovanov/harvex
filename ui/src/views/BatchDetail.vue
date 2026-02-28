<template>
  <div>
    <div class="d-flex align-center mb-4">
      <v-btn icon variant="text" to="/" class="mr-2">
        <v-icon>mdi-arrow-left</v-icon>
      </v-btn>
      <h1 class="text-h5">{{ batchesStore.current?.name || 'Batch Detail' }}</h1>
      <v-spacer />

      <v-btn-group v-if="batchesStore.current" variant="outlined" density="compact">
        <v-btn
          v-if="batchesStore.current.status === 'pending'"
          color="primary"
          prepend-icon="mdi-play"
          @click="startProcessing"
        >
          Process
        </v-btn>
        <v-menu>
          <template #activator="{ props }">
            <v-btn v-bind="props" prepend-icon="mdi-download">Export</v-btn>
          </template>
          <v-list density="compact">
            <v-list-item :href="jsonUrl" target="_blank">
              <template #prepend><v-icon>mdi-code-json</v-icon></template>
              <v-list-item-title>JSON</v-list-item-title>
            </v-list-item>
            <v-list-item :href="excelUrl" target="_blank">
              <template #prepend><v-icon>mdi-file-excel</v-icon></template>
              <v-list-item-title>Excel</v-list-item-title>
            </v-list-item>
            <v-list-item :href="csvUrl" target="_blank">
              <template #prepend><v-icon>mdi-file-delimited</v-icon></template>
              <v-list-item-title>CSV</v-list-item-title>
            </v-list-item>
          </v-list>
        </v-menu>
        <v-btn color="error" icon="mdi-delete" @click="confirmDelete" />
      </v-btn-group>
    </div>

    <v-progress-linear v-if="loading" indeterminate color="primary" class="mb-4" />
    <v-alert v-if="batchesStore.error" type="error" closable class="mb-4">
      {{ batchesStore.error }}
    </v-alert>

    <!-- Progress -->
    <BatchProgress
      v-if="batchesStore.current"
      :batch="batchesStore.current"
      :progress="batchesStore.getProgress(batchId)"
    />

    <!-- Tabs -->
    <v-tabs v-model="tab" class="mb-4">
      <v-tab value="extractions">
        Extractions
        <v-badge
          v-if="extractionsStore.extractions.length > 0"
          :content="extractionsStore.extractions.length"
          color="primary"
          inline
          class="ml-1"
        />
      </v-tab>
      <v-tab value="documents">
        Documents
        <v-badge
          v-if="documentsStore.documents.length > 0"
          :content="documentsStore.documents.length"
          color="primary"
          inline
          class="ml-1"
        />
      </v-tab>
    </v-tabs>

    <v-tabs-window v-model="tab">
      <!-- Extractions tab -->
      <v-tabs-window-item value="extractions">
        <div v-if="extractionsStore.extractions.length === 0 && !extractionsStore.loading">
          <v-alert type="info" variant="tonal">
            No extractions yet.
            <span v-if="batchesStore.current?.status === 'pending'">
              Start processing to extract data from documents.
            </span>
            <span v-else-if="batchesStore.current?.status === 'processing'">
              Processing in progress...
            </span>
          </v-alert>
        </div>

        <div v-for="ext in extractionsStore.extractions" :key="ext.id">
          <ExtractionCard
            :extraction="ext"
            :document-name="getDocName(ext.document_id)"
            @view-details="viewExtraction(ext)"
          />
        </div>
      </v-tabs-window-item>

      <!-- Documents tab -->
      <v-tabs-window-item value="documents">
        <v-table v-if="documentsStore.documents.length > 0" density="compact">
          <thead>
            <tr>
              <th>Name</th>
              <th>Type</th>
              <th>Size</th>
              <th>Status</th>
              <th style="width: 80px"></th>
            </tr>
          </thead>
          <tbody>
            <tr v-for="doc in documentsStore.documents" :key="doc.id">
              <td>
                <v-icon size="small" class="mr-1">{{ fileIcon(doc.content_type) }}</v-icon>
                {{ doc.original_name }}
              </td>
              <td class="text-caption">{{ doc.content_type }}</td>
              <td class="text-caption">{{ documentsStore.formatFileSize(doc.file_size) }}</td>
              <td>
                <v-chip :color="docStatusColor(doc.status)" size="x-small" variant="tonal">
                  {{ doc.status }}
                </v-chip>
              </td>
              <td>
                <v-btn
                  icon
                  size="x-small"
                  variant="text"
                  color="error"
                  @click="deleteDoc(doc.id)"
                >
                  <v-icon size="small">mdi-delete</v-icon>
                </v-btn>
              </td>
            </tr>
          </tbody>
        </v-table>
        <v-alert v-else type="info" variant="tonal">
          No documents in this batch.
        </v-alert>
      </v-tabs-window-item>
    </v-tabs-window>

    <!-- Extraction detail dialog -->
    <v-dialog v-model="detailDialog" max-width="700">
      <v-card v-if="selectedExtraction">
        <v-card-title class="d-flex align-center">
          Extraction Detail
          <v-spacer />
          <v-btn icon variant="text" @click="detailDialog = false">
            <v-icon>mdi-close</v-icon>
          </v-btn>
        </v-card-title>
        <v-card-text>
          <ExtractionCard :extraction="selectedExtraction" :show-actions="false" />
          <v-divider class="my-3" />
          <h3 class="text-subtitle-2 mb-2">Full Structured Data</h3>
          <pre class="text-body-2 pa-3 bg-grey-darken-4 rounded" style="overflow-x: auto; white-space: pre-wrap">{{ JSON.stringify(selectedExtraction.structured_data, null, 2) }}</pre>
          <div v-if="selectedExtraction.raw_text" class="mt-3">
            <h3 class="text-subtitle-2 mb-2">Raw Text</h3>
            <v-textarea
              :model-value="selectedExtraction.raw_text"
              variant="outlined"
              readonly
              rows="8"
              density="compact"
            />
          </div>
        </v-card-text>
      </v-card>
    </v-dialog>

    <!-- Delete confirmation -->
    <v-dialog v-model="deleteDialog" max-width="400">
      <v-card>
        <v-card-title>Delete Batch</v-card-title>
        <v-card-text>
          Are you sure? This will permanently delete this batch and all its documents and extractions.
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
import { ref, computed, onMounted, watch } from 'vue'
import { useRoute, useRouter } from 'vue-router'
import { useBatchesStore } from '../stores/batches'
import { useDocumentsStore } from '../stores/documents'
import { useExtractionsStore } from '../stores/extractions'
import { useSnackbar } from '../composables/useSnackbar'
import { api, type Extraction } from '../api/client'
import BatchProgress from '../components/BatchProgress.vue'
import ExtractionCard from '../components/ExtractionCard.vue'

const route = useRoute()
const router = useRouter()
const batchesStore = useBatchesStore()
const documentsStore = useDocumentsStore()
const extractionsStore = useExtractionsStore()
const { showSuccess, showError } = useSnackbar()

const batchId = computed(() => route.params.id as string)
const tab = ref('extractions')
const loading = ref(false)
const deleteDialog = ref(false)
const deleting = ref(false)
const detailDialog = ref(false)
const selectedExtraction = ref<Extraction | null>(null)

const jsonUrl = computed(() => api.exportJsonUrl(batchId.value))
const excelUrl = computed(() => api.exportExcelUrl(batchId.value))
const csvUrl = computed(() => api.exportCsvUrl(batchId.value))

async function loadData() {
  loading.value = true
  await Promise.all([
    batchesStore.fetchOne(batchId.value),
    documentsStore.fetchByBatch(batchId.value),
    extractionsStore.fetchByBatch(batchId.value),
  ])
  loading.value = false

  if (batchesStore.current?.status === 'processing') {
    batchesStore.startProgressStream(batchId.value)
  }
}

onMounted(loadData)

watch(
  () => batchesStore.current?.status,
  (newStatus, oldStatus) => {
    if (oldStatus === 'processing' && newStatus !== 'processing') {
      extractionsStore.fetchByBatch(batchId.value)
      documentsStore.fetchByBatch(batchId.value)
    }
  },
)

function getDocName(docId: string): string {
  return documentsStore.documents.find((d) => d.id === docId)?.original_name || docId
}

async function startProcessing() {
  try {
    await batchesStore.process(batchId.value)
    batchesStore.startProgressStream(batchId.value)
    showSuccess('Processing started')
  } catch (e) {
    showError((e as Error).message)
  }
}

function viewExtraction(ext: Extraction) {
  selectedExtraction.value = ext
  detailDialog.value = true
}

async function deleteDoc(docId: string) {
  try {
    await documentsStore.deleteDocument(docId)
    showSuccess('Document deleted')
  } catch (e) {
    showError((e as Error).message)
  }
}

function confirmDelete() {
  deleteDialog.value = true
}

async function doDelete() {
  deleting.value = true
  try {
    await batchesStore.deleteBatch(batchId.value)
    showSuccess('Batch deleted')
    router.push('/')
  } catch (e) {
    showError((e as Error).message)
  } finally {
    deleting.value = false
  }
}

function fileIcon(contentType: string): string {
  if (contentType.includes('pdf')) return 'mdi-file-pdf-box'
  if (contentType.startsWith('image/')) return 'mdi-file-image'
  if (contentType.includes('word') || contentType.includes('document')) return 'mdi-file-word'
  if (contentType.includes('sheet') || contentType.includes('excel')) return 'mdi-file-excel'
  return 'mdi-file'
}

function docStatusColor(status: string): string {
  switch (status) {
    case 'completed': case 'extracted': return 'success'
    case 'processing': return 'primary'
    case 'failed': case 'error': return 'error'
    default: return 'grey'
  }
}
</script>
