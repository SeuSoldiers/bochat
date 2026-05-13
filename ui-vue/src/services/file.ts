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

  return apiClient.post<UploadedFile>('/file/upload', formData, {
    headers: {
      'Content-Type': 'multipart/form-data',
      Authorization: `Bearer ${botToken}`,
    },
  })
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
