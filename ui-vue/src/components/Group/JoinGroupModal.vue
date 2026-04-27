<template>
  <div class="modal-overlay" @click="$emit('close')">
    <div class="modal-content" @click.stop>
      <div class="modal-header">
        <h2>加入群组</h2>
        <button class="close-btn" @click="$emit('close')">✕</button>
      </div>

      <form @submit.prevent="handleSubmit" class="modal-form">
        <div class="form-group">
          <label for="group-search">按群号搜索群聊</label>
          <div class="search-row">
            <input
              id="group-search"
              v-model="searchGroupCode"
              type="text"
              placeholder="可留空搜索公开群"
              :disabled="loading || searching"
            />
            <button type="button" class="btn-search" :disabled="loading || searching" @click="handleSearch">
              {{ searching ? '搜索中' : '搜索' }}
            </button>
          </div>
          <p class="search-hint">默认下方列出你自己的群聊，也可留空搜索公开群。</p>
        </div>

        <div v-if="searchError" class="error-message">
          {{ searchError }}
        </div>

        <div v-if="searchedGroups.length > 0" class="result-block">
          <p class="block-title">搜索结果</p>
          <div class="group-list">
            <button
              v-for="group in searchedGroups"
              :key="group.group_id"
              type="button"
              :class="['group-option', { selected: selectedKey === `search:${group.group_id}` }]"
              @click="selectSearchedGroup(group)"
            >
              <span class="group-name">{{ group.name }}</span>
              <span class="group-meta">群号：{{ group.group_code || '未设置' }}</span>
            </button>
          </div>
        </div>

        <div class="result-block">
          <p class="block-title">我的群聊</p>
          <div v-if="groups.length > 0" class="group-list">
            <button
              v-for="group in groups"
              :key="group.group_id"
              type="button"
              :class="['group-option', { selected: selectedKey === `own:${group.group_id}` }]"
              @click="selectOwnGroup(group)"
            >
              <span class="group-name">{{ group.name }}</span>
              <span class="group-meta">群号：{{ group.group_code || '未设置' }}</span>
              <span class="group-meta">状态：{{ group.status }}</span>
            </button>
          </div>
          <p v-else class="empty-tip">暂无可选群聊，请先创建群聊或先搜索群号。</p>
        </div>

        <div class="form-group">
          <label for="join-reason">申请理由</label>
          <textarea
            id="join-reason"
            v-model="requestReason"
            placeholder="请填写申请加入群聊的理由"
            :disabled="loading"
            rows="3"
          />
        </div>

        <div v-if="error" class="error-message">
          {{ error }}
        </div>

        <div class="form-actions">
          <button type="button" class="btn-cancel" @click="$emit('close')" :disabled="loading">
            取消
          </button>
          <button type="submit" class="btn-submit" :disabled="loading">
            {{ loading ? '加入中...' : '加入群聊' }}
          </button>
        </div>
      </form>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref } from 'vue'
import type { Group } from '@/types'
import { getErrorMessage } from '@/utils/error'
import { searchGroupByCode } from '@/services/group'

const props = defineProps<{
  groups: Group[]
  preselectedBotId?: string
}>()

const emit = defineEmits<{
  join: [payload: { groupId?: string; groupCode?: string; botId: string; requestReason: string }]
  close: []
}>()

const loading = ref(false)
const searching = ref(false)
const error = ref<string | null>(null)
const searchError = ref<string | null>(null)
const searchGroupCode = ref('')
const searchedGroups = ref<Group[]>([])
const selectedKey = ref('')
const selectedPayload = ref<{ groupId?: string; groupCode?: string } | null>(null)
const requestReason = ref('')

const selectOwnGroup = (group: Group) => {
  selectedKey.value = `own:${group.group_id}`
  selectedPayload.value = {
    groupId: group.group_id,
    groupCode: group.group_code,
  }
  error.value = null
}

const selectSearchedGroup = (group: Group) => {
  selectedKey.value = `search:${group.group_id}`
  selectedPayload.value = {
    groupId: group.group_id,
    groupCode: group.group_code,
  }
  error.value = null
}

const handleSearch = async () => {
  const groupCode = searchGroupCode.value.trim()

  searching.value = true
  searchError.value = null
  searchedGroups.value = []

  try {
    const foundGroups = await searchGroupByCode(groupCode)
    searchedGroups.value = foundGroups
    if (foundGroups.length === 0) {
      searchError.value = '未找到匹配的群聊'
    }
  } catch (err: any) {
    searchError.value = getErrorMessage(err, '搜索群聊失败')
  } finally {
    searching.value = false
  }
}

const handleSubmit = async () => {
  if (!props.preselectedBotId) {
    error.value = '当前机器人无效，请关闭后重试'
    return
  }

  if (!selectedPayload.value) {
    error.value = '请选择一个群聊'
    return
  }

  const reason = requestReason.value.trim()
  if (!reason) {
    error.value = '请填写申请理由'
    return
  }

  loading.value = true
  error.value = null

  try {
    emit('join', {
      ...selectedPayload.value,
      botId: props.preselectedBotId,
      requestReason: reason,
    })
  } catch (err: any) {
    error.value = getErrorMessage(err, '加入失败')
  } finally {
    loading.value = false
  }
}
</script>

<style scoped>
.modal-overlay {
  position: fixed;
  inset: 0;
  background-color: rgba(0, 0, 0, 0.28);
  display: flex;
  align-items: center;
  justify-content: center;
  z-index: 1000;
}

.modal-content {
  width: 100%;
  max-width: 560px;
  border-radius: 8px;
  background: #f5f5f5;
  border: 1px solid #d0d0d0;
  box-shadow: 0 6px 18px rgba(0, 0, 0, 0.12);
}

.modal-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 16px 18px;
  border-bottom: 1px solid #d0d0d0;
}

.modal-header h2 {
  margin: 0;
  font-size: 16px;
  color: #2f2f2f;
}

.close-btn {
  border: none;
  background: transparent;
  color: #7a7a7a;
  font-size: 18px;
  cursor: pointer;
}

.modal-form {
  padding: 16px 18px;
}

.form-group {
  margin-bottom: 14px;
}

.form-group label {
  display: block;
  margin-bottom: 6px;
  font-size: 13px;
  color: #4a4a4a;
  font-weight: 600;
}

.search-row {
  display: grid;
  grid-template-columns: minmax(0, 1fr) auto;
  gap: 8px;
}

.form-group input {
  width: 100%;
  border: 1px solid #d0d0d0;
  border-radius: 6px;
  padding: 10px 12px;
  font-size: 14px;
  color: #2f2f2f;
  background: #f7f7f7;
}

.form-group textarea {
  width: 100%;
  border: 1px solid #d0d0d0;
  border-radius: 6px;
  padding: 10px 12px;
  font-size: 14px;
  color: #2f2f2f;
  background: #f7f7f7;
  resize: vertical;
  min-height: 72px;
  font-family: inherit;
}

.btn-search {
  border: 1px solid #c0c0c0;
  background: #ececec;
  color: #2f2f2f;
  border-radius: 6px;
  padding: 0 12px;
  font-size: 13px;
  font-weight: 600;
  cursor: pointer;
}

.search-hint {
  margin: 8px 0 0;
  font-size: 12px;
  color: #7a7a7a;
}

.result-block {
  margin-bottom: 14px;
}

.block-title {
  margin: 0 0 8px;
  font-size: 13px;
  color: #4a4a4a;
  font-weight: 600;
}

.group-list {
  display: grid;
  grid-template-columns: repeat(2, minmax(0, 1fr));
  gap: 8px;
}

.group-option {
  width: 100%;
  border: 1px solid #d0d0d0;
  border-radius: 8px;
  background: #f7f7f7;
  padding: 10px;
  display: flex;
  flex-direction: column;
  gap: 4px;
  text-align: left;
  cursor: pointer;
}

.group-option.selected {
  background: #2f2f2f;
  border-color: #2f2f2f;
}

.group-name {
  font-size: 14px;
  font-weight: 700;
  color: #2f2f2f;
}

.group-meta {
  font-size: 12px;
  color: #666666;
}

.group-option.selected .group-name,
.group-option.selected .group-meta {
  color: #f3f3f3;
}

.empty-tip {
  margin: 0;
  padding: 10px 12px;
  border-radius: 8px;
  border: 1px dashed #d0d0d0;
  color: #7a7a7a;
  font-size: 12px;
}

.error-message {
  margin-bottom: 12px;
  padding: 9px 11px;
  border-radius: 6px;
  border: 1px solid #e0b7b7;
  background: #f3e7e7;
  color: #9b3c3c;
  font-size: 12px;
}

.form-actions {
  display: flex;
  gap: 8px;
  justify-content: flex-end;
}

.form-actions button {
  border-radius: 6px;
  padding: 9px 14px;
  font-size: 13px;
  font-weight: 600;
  border: 1px solid transparent;
  cursor: pointer;
}

.btn-cancel {
  border-color: #d0d0d0;
  background: #efefef;
  color: #2f2f2f;
}

.btn-submit {
  border-color: #2f2f2f;
  background: #2f2f2f;
  color: #f3f3f3;
}

.btn-cancel:disabled,
.btn-submit:disabled,
.btn-search:disabled {
  opacity: 0.6;
  cursor: not-allowed;
}

@media (max-width: 768px) {
  .modal-content {
    max-width: calc(100vw - 20px);
  }

  .group-list {
    grid-template-columns: 1fr;
  }
}
</style>
