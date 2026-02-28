<template>
  <v-card
    :class="['file-drop-zone', { 'drop-active': isDragOver }]"
    variant="outlined"
    @dragover.prevent="isDragOver = true"
    @dragleave.prevent="isDragOver = false"
    @drop.prevent="onDrop"
    @click="openFilePicker"
  >
    <v-card-text class="text-center pa-8">
      <v-icon size="64" :color="isDragOver ? 'primary' : 'grey'" class="mb-4">
        {{ isDragOver ? 'mdi-cloud-upload' : 'mdi-file-upload-outline' }}
      </v-icon>
      <div class="text-h6 mb-2">
        {{ isDragOver ? 'Drop files here' : 'Drag & drop files or click to browse' }}
      </div>
      <div class="text-body-2 text-medium-emphasis">
        Supported: PDF, Images (JPG, PNG, TIFF), Word (DOCX), Excel (XLSX, XLS)
      </div>
      <div v-if="selectedFiles.length > 0" class="mt-4">
        <v-chip
          v-for="(file, i) in selectedFiles"
          :key="i"
          closable
          class="ma-1"
          @click:close="removeFile(i)"
          @click.stop
        >
          <v-icon start size="small">{{ fileIcon(file) }}</v-icon>
          {{ file.name }}
          <span class="text-caption ml-1">({{ formatSize(file.size) }})</span>
        </v-chip>
      </div>
    </v-card-text>
    <input
      ref="fileInput"
      type="file"
      multiple
      :accept="acceptTypes"
      style="display: none"
      @change="onFileSelect"
    />
  </v-card>
</template>

<script setup lang="ts">
import { ref } from 'vue'

const emit = defineEmits<{
  (e: 'files-selected', files: File[]): void
}>()

const isDragOver = ref(false)
const selectedFiles = ref<File[]>([])
const fileInput = ref<HTMLInputElement>()

const acceptTypes = '.pdf,.jpg,.jpeg,.png,.tiff,.tif,.docx,.xlsx,.xls'

function onDrop(event: DragEvent) {
  isDragOver.value = false
  const files = Array.from(event.dataTransfer?.files || [])
  addFiles(files)
}

function openFilePicker() {
  fileInput.value?.click()
}

function onFileSelect(event: Event) {
  const input = event.target as HTMLInputElement
  const files = Array.from(input.files || [])
  addFiles(files)
  input.value = '' // reset to allow re-selecting same files
}

function addFiles(files: File[]) {
  selectedFiles.value.push(...files)
  emit('files-selected', selectedFiles.value)
}

function removeFile(index: number) {
  selectedFiles.value.splice(index, 1)
  emit('files-selected', selectedFiles.value)
}

function clear() {
  selectedFiles.value = []
}

function fileIcon(file: File): string {
  const type = file.type
  const name = file.name.toLowerCase()
  if (type === 'application/pdf' || name.endsWith('.pdf')) return 'mdi-file-pdf-box'
  if (type.startsWith('image/')) return 'mdi-file-image'
  if (name.endsWith('.docx') || name.endsWith('.doc')) return 'mdi-file-word'
  if (name.endsWith('.xlsx') || name.endsWith('.xls')) return 'mdi-file-excel'
  return 'mdi-file'
}

function formatSize(bytes: number): string {
  if (bytes < 1024) return `${bytes} B`
  if (bytes < 1024 * 1024) return `${(bytes / 1024).toFixed(1)} KB`
  return `${(bytes / (1024 * 1024)).toFixed(1)} MB`
}

defineExpose({ clear })
</script>

<style scoped>
.file-drop-zone {
  cursor: pointer;
  transition: all 0.2s ease;
  border-style: dashed !important;
  border-width: 2px !important;
}
.file-drop-zone:hover {
  border-color: rgb(var(--v-theme-primary)) !important;
}
.drop-active {
  border-color: rgb(var(--v-theme-primary)) !important;
  background: rgba(var(--v-theme-primary), 0.05);
}
</style>
