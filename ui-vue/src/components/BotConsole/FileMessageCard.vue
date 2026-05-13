<template>
  <div class="file-message-card" :class="{ 'is-image': isImage && fileUrl }">
    <div v-if="isImage && fileUrl" class="image-preview-wrap">
      <img :src="fileUrl" :alt="fileName" class="image-preview" loading="lazy" />
    </div>
    <div v-else class="file-icon-wrap">
      <FileText v-if="isDoc" class="file-icon doc" />
      <FileImage v-else-if="isImage" class="file-icon image" />
      <FileCode2 v-else-if="isCode" class="file-icon code" />
      <FileSpreadsheet v-else-if="isSheet" class="file-icon sheet" />
      <FileArchive v-else-if="isArchive" class="file-icon archive" />
      <FileVideo v-else-if="isVideo" class="file-icon video" />
      <FileAudio v-else-if="isAudio" class="file-icon audio" />
      <File v-else class="file-icon" />
    </div>
    <div class="file-body">
      <p class="file-name">{{ fileName }}</p>
      <p v-if="fileSize" class="file-size">{{ fileSize }}</p>
    </div>
    <button
      type="button"
      class="download-btn"
      :class="{ disabled: !canDownload }"
      :disabled="!canDownload"
      :title="canDownload ? '下载文件' : '无法获取下载地址'"
      @click="handleDownload"
    >
      <Download class="download-icon" />
    </button>
  </div>
</template>

<script setup lang="ts">
import { computed } from 'vue'
import {
  File,
  FileText,
  FileImage,
  FileCode2,
  FileSpreadsheet,
  FileArchive,
  FileVideo,
  FileAudio,
  Download,
} from 'lucide-vue-next'
import { downloadFile } from '@/services/file'
import { useBotConsoleStore } from '@/stores/botConsole'

const props = defineProps<{
  fileName: string
  fileSize?: string
  fileUrl?: string
}>()

const store = useBotConsoleStore()
const lower = computed(() => props.fileName.toLowerCase())

const isDoc = computed(() => /\.(pdf|doc|docx|odt|rtf|ppt|pptx|key|txt)$/.test(lower.value))
const isImage = computed(() => /\.(png|jpe?g|gif|webp|bmp|svg|ico)$/.test(lower.value))
const isCode = computed(() => /\.(js|ts|tsx|jsx|py|rs|go|java|c|cpp|h|hpp|json|yaml|yml|toml|md|sql)$/.test(lower.value))
const isSheet = computed(() => /\.(xls|xlsx|csv)$/.test(lower.value))
const isArchive = computed(() => /\.(zip|rar|7z|tar|gz)$/.test(lower.value))
const isVideo = computed(() => /\.(mp4|mov|mkv|avi|webm)$/.test(lower.value))
const isAudio = computed(() => /\.(mp3|wav|flac|aac|ogg)$/.test(lower.value))

const canDownload = computed(() => !!props.fileUrl)

const handleDownload = async () => {
  if (!props.fileUrl) return
  try {
    await downloadFile(props.fileUrl, props.fileName || 'download')
  } catch {
    store.notify('文件下载失败，请稍后重试')
  }
}
</script>

<style scoped>
.file-message-card {
  display: inline-flex;
  align-items: center;
  gap: 10px;
  padding: 10px 12px;
  background: #f5f5f5;
  border: 1px solid #d0d0d0;
  border-radius: 8px;
  max-width: 320px;
}

.file-message-card.is-image {
  flex-direction: column;
  align-items: flex-start;
  padding: 8px;
  gap: 6px;
  position: relative;
}

.image-preview-wrap {
  width: 100%;
  border-radius: 6px;
  overflow: hidden;
  border: 1px solid #d0d0d0;
}

.image-preview {
  display: block;
  width: 100%;
  height: auto;
  max-height: 240px;
  object-fit: cover;
  cursor: pointer;
  transition: transform 0.2s ease;
}

.image-preview:hover {
  transform: scale(1.02);
}

/* 图片模式下的底部信息栏 */
.file-message-card.is-image .file-body {
  padding: 2px 4px 0;
}

.file-message-card.is-image .file-name {
  font-size: 11px;
  color: #6f6f6f;
}

.file-message-card.is-image .download-btn {
  position: absolute;
  top: 12px;
  right: 12px;
  background: rgba(47, 47, 47, 0.7);
  color: #f3f3f3;
  border-radius: 4px;
}

.file-message-card.is-image .download-btn:hover:not(:disabled) {
  background: rgba(47, 47, 47, 0.85);
}

.file-icon-wrap {
  width: 36px;
  height: 36px;
  border-radius: 6px;
  display: grid;
  place-items: center;
  background: #ebebeb;
  flex-shrink: 0;
}

.file-icon {
  width: 18px;
  height: 18px;
  stroke: #4a4a4a;
  stroke-width: 2;
}

.file-icon.doc { stroke: #2f5f8f; }
.file-icon.image { stroke: #8f5f2f; }
.file-icon.code { stroke: #2f8f4e; }
.file-icon.sheet { stroke: #2f8f5f; }
.file-icon.archive { stroke: #6f4f8f; }
.file-icon.video { stroke: #8f2f4f; }
.file-icon.audio { stroke: #4f6f8f; }

.file-body {
  min-width: 0;
  flex: 1;
}

.file-name {
  margin: 0;
  font-size: 13px;
  color: #1f1f1f;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.file-size {
  margin: 2px 0 0;
  font-size: 11px;
  color: #6f6f6f;
}

.download-btn {
  width: 28px;
  height: 28px;
  border-radius: 6px;
  border: none;
  background: transparent;
  color: #6f6f6f;
  display: grid;
  place-items: center;
  cursor: pointer;
  transition: var(--transition-base, all 0.2s ease);
  flex-shrink: 0;
}

.download-btn:hover:not(:disabled) {
  background: #ebebeb;
  color: #1f1f1f;
}

.download-btn:disabled {
  opacity: 0.4;
  cursor: not-allowed;
}

.download-icon {
  width: 16px;
  height: 16px;
  stroke: currentColor;
  stroke-width: 2;
}
</style>
