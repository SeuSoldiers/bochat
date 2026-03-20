import { apiClient } from './api'

export interface UploadedFile {
  file_id: string
  url: string
  created_at: string
}

export async function uploadFile(file: File) {
  const formData = new FormData()
  formData.append('file', file)

  return apiClient.post<UploadedFile>('/file/upload', formData, {
    headers: {
      'Content-Type': 'multipart/form-data',
    },
  })
}
