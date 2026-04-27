<template>
  <div class="page-shell audit-page">
    <div class="shell-body">
      <TopNav />

      <main class="audit-main">
        <section class="audit-card dashboard-surface">
          <header class="audit-header">
            <h2>审计日志</h2>
            <button class="btn btn-primary" :disabled="loading || exporting" @click="handleExport">
              {{ exporting ? '导出中...' : '导出 CSV' }}
            </button>
          </header>

          <form class="filters" @submit.prevent="handleSearch">
            <input v-model.trim="filters.user_id" type="text" placeholder="用户编号" />
            <input v-model.trim="filters.bot_id" type="text" placeholder="Bot 编号" />
            <input v-model.trim="filters.group_id" type="text" placeholder="群编号" />
            <input v-model.trim="filters.action" type="text" placeholder="动作，如 group.create" />
            <input v-model="filters.start_at" type="datetime-local" />
            <input v-model="filters.end_at" type="datetime-local" />
            <div class="filter-actions">
              <button type="button" class="btn btn-secondary" :disabled="loading" @click="handleReset">
                重置
              </button>
              <button type="submit" class="btn btn-primary" :disabled="loading">
                {{ loading ? '查询中...' : '查询' }}
              </button>
            </div>
          </form>

          <p v-if="error" class="error-text">{{ error }}</p>

          <div class="table-wrap">
            <table class="audit-table">
              <thead>
                <tr>
                  <th>时间</th>
                  <th>动作</th>
                  <th>操作者</th>
                  <th>目标</th>
                  <th>详情</th>
                </tr>
              </thead>
              <tbody>
                <tr v-if="!loading && logs.length === 0">
                  <td colspan="5" class="empty-cell">暂无日志</td>
                </tr>
                <tr v-for="log in logs" :key="log.log_id">
                  <td>{{ formatTime(log.created_at) }}</td>
                  <td><code>{{ log.action }}</code></td>
                  <td>{{ log.actor_type }} / {{ log.actor_id }}</td>
                  <td>{{ formatTarget(log) }}</td>
                  <td class="details-cell" :title="formatDetails(log.details)">
                    {{ formatDetailsPreview(log.details) }}
                  </td>
                </tr>
              </tbody>
            </table>
          </div>
        </section>
      </main>
    </div>
  </div>
</template>

<script setup lang="ts">
import { onMounted, reactive, ref } from 'vue'
import TopNav from '@/components/Common/TopNav.vue'
import { exportAuditLogsCsv, getAuditLogs, type AuditLogItem, type AuditLogQuery } from '@/services/audit'
import { getErrorMessage } from '@/utils/error'

const logs = ref<AuditLogItem[]>([])
const loading = ref(false)
const exporting = ref(false)
const error = ref<string | null>(null)

const filters = reactive<AuditLogQuery>({
  user_id: '',
  bot_id: '',
  group_id: '',
  action: '',
  start_at: '',
  end_at: '',
  limit: 100,
  offset: 0,
})

const toIso = (value?: string) => {
  if (!value) {
    return undefined
  }
  const date = new Date(value)
  if (Number.isNaN(date.getTime())) {
    return undefined
  }
  return date.toISOString()
}

const buildQuery = (): AuditLogQuery => ({
  user_id: filters.user_id || undefined,
  bot_id: filters.bot_id || undefined,
  group_id: filters.group_id || undefined,
  action: filters.action || undefined,
  start_at: toIso(filters.start_at),
  end_at: toIso(filters.end_at),
  limit: filters.limit || 100,
  offset: filters.offset || 0,
})

const fetchLogs = async () => {
  loading.value = true
  error.value = null

  try {
    const response = await getAuditLogs(buildQuery())
    logs.value = response.logs || []
  } catch (err: any) {
    error.value = getErrorMessage(err, '获取审计日志失败')
  } finally {
    loading.value = false
  }
}

const handleSearch = async () => {
  filters.offset = 0
  await fetchLogs()
}

const handleReset = async () => {
  filters.user_id = ''
  filters.bot_id = ''
  filters.group_id = ''
  filters.action = ''
  filters.start_at = ''
  filters.end_at = ''
  filters.offset = 0
  await fetchLogs()
}

const handleExport = async () => {
  exporting.value = true
  error.value = null

  try {
    const blob = await exportAuditLogsCsv(buildQuery())
    const url = window.URL.createObjectURL(blob)
    const link = document.createElement('a')
    link.href = url
    link.download = 'audit_logs.csv'
    document.body.appendChild(link)
    link.click()
    document.body.removeChild(link)
    window.URL.revokeObjectURL(url)
  } catch (err: any) {
    error.value = getErrorMessage(err, '导出审计日志失败')
  } finally {
    exporting.value = false
  }
}

const formatTime = (value: string) => {
  if (!value) {
    return '-'
  }
  const date = new Date(value)
  if (Number.isNaN(date.getTime())) {
    return value
  }
  return date.toLocaleString()
}

const formatTarget = (log: AuditLogItem) => {
  if (log.user_id) {
    return `user:${log.user_id}`
  }
  if (log.bot_id) {
    return `bot:${log.bot_id}`
  }
  if (log.group_id) {
    return `group:${log.group_id}`
  }
  if (log.resource_id) {
    return `${log.resource_type}:${log.resource_id}`
  }
  return '-'
}

const formatDetails = (details: AuditLogItem['details']) => {
  if (!details) {
    return '-'
  }
  if (typeof details === 'string') {
    return details
  }
  return JSON.stringify(details)
}

const formatDetailsPreview = (details: AuditLogItem['details']) => {
  const raw = formatDetails(details).replace(/\s+/g, ' ').trim()
  if (raw.length <= 120) {
    return raw
  }
  return `${raw.slice(0, 120)}...`
}

onMounted(fetchLogs)
</script>

<style scoped>
.audit-page {
  min-height: calc(100vh - 40px);
}

.shell-body {
  height: 100%;
  min-height: 0;
  display: flex;
  gap: 20px;
}

.audit-main {
  flex: 1;
  min-width: 0;
  min-height: 0;
  overflow: auto;
  padding: 8px;
}

.audit-card {
  display: flex;
  flex-direction: column;
  gap: 14px;
  padding: 18px;
  min-height: 100%;
}

.audit-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 12px;
}

.audit-header h2 {
  margin: 0;
  font-size: 22px;
  color: #1f1f1f;
}

.filters {
  display: grid;
  grid-template-columns: repeat(3, minmax(0, 1fr));
  gap: 10px;
}

.filters input {
  width: 100%;
  min-width: 0;
  border: 1px solid #cfcfcf;
  border-radius: 8px;
  padding: 8px 10px;
  background: #f5f5f5;
  color: #242424;
  font-size: 13px;
}

.filters input:focus {
  outline: none;
  border-color: #6f6f6f;
}

.filter-actions {
  grid-column: 1 / -1;
  display: flex;
  justify-content: flex-end;
  gap: 8px;
}

.btn {
  border-radius: 999px;
  border: 1px solid transparent;
  padding: 8px 14px;
  font-size: 12px;
  font-weight: 700;
  line-height: 1;
}

.btn:disabled {
  opacity: 0.7;
  cursor: not-allowed;
}

.btn-primary {
  background: #2f2f2f;
  border-color: #2f2f2f;
  color: #f3f3f3;
}

.btn-secondary {
  background: #ededed;
  border-color: #d2d2d2;
  color: #353535;
}

.error-text {
  color: #a6453e;
  font-size: 13px;
}

.table-wrap {
  overflow: auto;
  border: 1px solid #d8d8d8;
  border-radius: 10px;
}

.audit-table {
  width: 100%;
  border-collapse: collapse;
  font-size: 12px;
  color: #232323;
}

.audit-table th,
.audit-table td {
  border-bottom: 1px solid #e1e1e1;
  padding: 6px 10px;
  text-align: left;
  vertical-align: top;
  line-height: 1.3;
}

.audit-table th {
  background: #efefef;
  color: #2c2c2c;
  font-weight: 700;
  position: sticky;
  top: 0;
  z-index: 1;
}

.audit-table td code {
  background: #ebebeb;
  border-radius: 5px;
  padding: 1px 6px;
}

.details-cell {
  max-width: 360px;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

.empty-cell {
  text-align: center;
  color: #808080;
}

@media (max-width: 1024px) {
  .shell-body {
    flex-direction: column;
    gap: 12px;
  }

  .filters {
    grid-template-columns: repeat(2, minmax(0, 1fr));
  }
}
</style>
