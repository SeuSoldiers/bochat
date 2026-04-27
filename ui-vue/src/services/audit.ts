import { apiClient } from '@/services/api'

export interface AuditLogItem {
  log_id: number
  actor_type: string
  actor_id: string
  user_id?: string
  bot_id?: string
  group_id?: string
  action: string
  resource_type: string
  resource_id?: string
  details?: Record<string, unknown> | string
  created_at: string
}

export interface AuditLogQuery {
  user_id?: string
  bot_id?: string
  group_id?: string
  action?: string
  start_at?: string
  end_at?: string
  limit?: number
  offset?: number
}

export interface AuditLogsResponse {
  logs: AuditLogItem[]
  limit: number
  offset: number
}

function compactQuery(query: AuditLogQuery): Record<string, string | number> {
  const result: Record<string, string | number> = {}
  Object.entries(query).forEach(([key, value]) => {
    if (value !== undefined && value !== null && value !== '') {
      result[key] = value
    }
  })
  return result
}

export async function getAuditLogs(query: AuditLogQuery) {
  const params = compactQuery(query)
  return apiClient.get<AuditLogsResponse>('/audit/logs', { params })
}

export async function exportAuditLogsCsv(query: AuditLogQuery): Promise<Blob> {
  const params = compactQuery(query)
  return apiClient.get<Blob>('/audit/logs/export', {
    params,
    responseType: 'blob',
  })
}
