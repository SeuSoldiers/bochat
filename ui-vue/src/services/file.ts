import { apiClient } from './api'

export interface UploadedFile {
  file_id: string
  filename: string
  url: string
  created_at: string
}

export async function uploadFile(file: File, botToken: string) {
  const formData = new FormData()
  formData.append('file', file)

  const result = await apiClient.post<UploadedFile>('/file/upload', formData, {
    headers: {
      'Content-Type': 'multipart/form-data',
      Authorization: `Bearer ${botToken}`,
    },
  })
  // 后端返回相对路径，前端拼接完整URL
  if (result.url && result.url.startsWith('/')) {
    result.url = window.location.origin + result.url
  }
  return result
}

export async function downloadFile(fileUrl: string, fileName: string): Promise<void> {
  const response = await fetch(fileUrl)
  if (!response.ok) {
    throw new Error(`下载失败 (${response.status})`)
  }
  const blob = await response.blob()
  const blobUrl = URL.createObjectURL(blob)
  const anchor = document.createElement('a')
  anchor.href = blobUrl
  anchor.download = fileName
  document.body.appendChild(anchor)
  anchor.click()
  document.body.removeChild(anchor)
  URL.revokeObjectURL(blobUrl)
}
