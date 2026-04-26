<template>
  <div class="modal-overlay" @click="$emit('close')">
    <div class="modal-content" @click.stop>
      <div class="modal-header">
        <h2>群成员</h2>
        <button class="close-btn" @click="$emit('close')">✕</button>
      </div>

      <div class="modal-body">
        <div class="group-info section-card">
          <p class="group-name">{{ group.name }}</p>
          <p class="group-number">群号：{{ group.group_code || '未设置' }}</p>
        </div>

        <div class="toolbar section-card">
          <button class="toolbar-btn" :disabled="!canAddSelectedBot || loading" @click="handleAddBot">
            {{ loading ? '处理中...' : '邀请加入' }}
          </button>
          <button class="secondary-btn" :disabled="!canRemoveSelectedBot || loading" @click="handleRemoveBot">
            {{ loading ? '处理中...' : '移出群聊' }}
          </button>
        </div>

        <div class="search-block section-card">
          <label for="bot-search">按 Bot 编号搜索（精确匹配）</label>
          <div class="search-row">
            <input
              id="bot-search"
              v-model="searchBotId"
              type="text"
              placeholder="输入完整 Bot 编号后搜索"
              :disabled="loading || searching"
            />
            <button type="button" class="btn-search" :disabled="loading || searching" @click="handleSearch">
              {{ searching ? '搜索中' : '搜索' }}
            </button>
          </div>
        </div>

        <div v-if="searchError" class="error-message">
          {{ searchError }}
        </div>

        <div v-if="searchedBots.length > 0" class="result-block section-card">
          <p class="block-title">搜索结果</p>
          <div class="bot-list">
            <button
              v-for="bot in searchedBots"
              :key="bot.bot_id"
              type="button"
              :class="['bot-option', { selected: selectedKey === `search:${bot.bot_id}` }]"
              @click="selectSearchBot(bot.bot_id)"
            >
              <span class="bot-name">{{ bot.name }}</span>
              <span class="bot-meta">编号：{{ bot.bot_id }}</span>
              <span class="bot-meta">状态：{{ bot.status }}</span>
            </button>
          </div>
        </div>

        <div class="result-block section-card">
          <p class="block-title">我的 Bot</p>
          <div v-if="ownedBots.length > 0" class="bot-list">
            <button
              v-for="bot in ownedBots"
              :key="bot.bot_id"
              type="button"
              :class="['bot-option', { selected: selectedKey === `own:${bot.bot_id}` }]"
              @click="selectOwnBot(bot.bot_id)"
            >
              <span class="bot-name">{{ bot.name }}</span>
              <span class="bot-meta">编号：{{ bot.bot_id }}</span>
              <span class="bot-meta">状态：{{ bot.status }}</span>
            </button>
          </div>
          <p v-else class="empty-tip">你当前没有可邀请的 Bot。</p>
        </div>

        <div class="result-block section-card">
          <p class="block-title">当前成员</p>
          <div v-if="members.length > 0" class="member-list">
            <button
              v-for="member in members"
              :key="member.member_id"
              type="button"
              :class="['member-item', { selected: selectedBotId === member.member_id }]"
              @click="selectMember(member.member_id)"
            >
              <span class="member-name">{{ member.bot_name || member.member_type }}</span>
              <span class="member-meta">编号：{{ member.member_id }}</span>
              <span class="member-meta">加入时间：{{ formatDate(member.joined_at) }}</span>
            </button>
          </div>
          <p v-else class="empty-tip">暂无成员</p>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { computed, ref } from 'vue'
import type { Group, GroupMember, Bot } from '@/types'
import { searchBotById } from '@/services/bot'
import { getErrorMessage } from '@/utils/error'

const props = defineProps<{
  group: Group
  members: GroupMember[]
  ownedBots: Bot[]
  loading?: boolean
}>()

const emit = defineEmits<{
  'add-bot': [botId: string]
  'remove-bot': [botId: string]
  close: []
}>()

const selectedBotId = ref('')
const selectedKey = ref('')
const searchBotId = ref('')
const searching = ref(false)
const searchError = ref<string | null>(null)
const searchedBots = ref<Array<{ bot_id: string; name: string; status: string }>>([])

const canAddSelectedBot = computed(() => {
  if (!selectedBotId.value) {
    return false
  }
  return !props.members.some((member) => member.member_id === selectedBotId.value)
})

const canRemoveSelectedBot = computed(() => {
  if (!selectedBotId.value) {
    return false
  }
  return props.members.some((member) => member.member_id === selectedBotId.value)
})

const selectOwnBot = (botId: string) => {
  selectedBotId.value = botId
  selectedKey.value = `own:${botId}`
}

const selectSearchBot = (botId: string) => {
  selectedBotId.value = botId
  selectedKey.value = `search:${botId}`
}

const selectMember = (botId: string) => {
  selectedBotId.value = botId
  selectedKey.value = `member:${botId}`
}

const handleSearch = async () => {
  const botId = searchBotId.value.trim()
  if (!botId) {
    searchError.value = '请输入 Bot 编号'
    searchedBots.value = []
    return
  }

  searching.value = true
  searchError.value = null
  searchedBots.value = []

  try {
    const result = await searchBotById(botId)
    searchedBots.value = result.map((bot) => ({
      bot_id: bot.bot_id,
      name: bot.name,
      status: bot.status,
    }))
    if (searchedBots.value.length === 0) {
      searchError.value = '未找到该 Bot'
    }
  } catch (err: any) {
    searchError.value = getErrorMessage(err, '搜索 Bot 失败')
  } finally {
    searching.value = false
  }
}

const handleAddBot = () => {
  if (!selectedBotId.value) {
    return
  }
  emit('add-bot', selectedBotId.value)
}

const handleRemoveBot = () => {
  if (!selectedBotId.value) {
    return
  }
  emit('remove-bot', selectedBotId.value)
}

const formatDate = (dateStr: string) => {
  const date = new Date(dateStr)
  return date.toLocaleString('zh-CN', {
    month: '2-digit',
    day: '2-digit',
    hour: '2-digit',
    minute: '2-digit',
  })
}
</script>

<style scoped>
.modal-overlay {
  position: fixed;
  inset: 0;
  background-color: rgba(0, 0, 0, 0.3);
  display: flex;
  align-items: center;
  justify-content: center;
  z-index: 1000;
  padding: 16px;
}

.modal-content {
  width: 100%;
  max-width: 620px;
  max-height: 78vh;
  overflow: hidden;
  display: flex;
  flex-direction: column;
  background: #f5f5f5;
  border: 1px solid #d0d0d0;
  border-radius: 8px;
  box-shadow: 0 8px 18px rgba(0, 0, 0, 0.12);
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
  font-weight: 600;
  color: #2f2f2f;
}

.close-btn {
  background: transparent;
  border: none;
  width: 30px;
  height: 30px;
  border-radius: 6px;
  font-size: 18px;
  color: #6f6f6f;
  cursor: pointer;
}

.modal-body {
  flex: 1;
  overflow-y: auto;
  padding: 16px 18px 18px;
  display: flex;
  flex-direction: column;
  gap: 10px;
}

.group-info {
  margin: 0;
}

.group-name {
  margin: 0 0 4px 0;
  font-size: 15px;
  font-weight: 700;
  color: #2f2f2f;
}

.group-number {
  margin: 0;
  font-size: 12px;
  color: #6f6f6f;
}

.toolbar {
  display: flex;
  gap: 8px;
  margin: 0;
}

.toolbar-btn,
.secondary-btn {
  padding: 8px 12px;
  border: 1px solid #d0d0d0;
  border-radius: 6px;
  font-size: 13px;
  font-weight: 600;
  cursor: pointer;
}

.toolbar-btn {
  background: #2f2f2f;
  color: #f3f3f3;
  border-color: #2f2f2f;
}

.secondary-btn {
  background: #efefef;
  color: #2f2f2f;
}

.toolbar-btn:disabled,
.secondary-btn:disabled {
  opacity: 0.6;
  cursor: not-allowed;
}

.search-block {
  margin: 0;
}

.search-block label {
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

.search-row input {
  border: 1px solid #d0d0d0;
  border-radius: 6px;
  padding: 10px 12px;
  font-size: 14px;
  color: #2f2f2f;
  background: #f7f7f7;
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

.result-block {
  margin: 0;
}

.block-title {
  margin: 0 0 8px;
  font-size: 13px;
  color: #4a4a4a;
  font-weight: 600;
}

.bot-list,
.member-list {
  display: grid;
  grid-template-columns: repeat(2, minmax(0, 1fr));
  gap: 10px;
}

.bot-option,
.member-item {
  border: 1px solid #d0d0d0;
  border-radius: 8px;
  background: #f5f5f5;
  color: #2f2f2f;
  text-align: left;
  padding: 12px;
  cursor: pointer;
  display: flex;
  flex-direction: column;
  gap: 6px;
  transition: var(--transition-base);
}

.bot-option.selected,
.member-item.selected {
  background: #2f2f2f;
  border-color: #2f2f2f;
  color: #f3f3f3;
}

.bot-option:hover,
.member-item:hover {
  border-color: #bcbcbc;
  box-shadow: 0 4px 10px rgba(0, 0, 0, 0.05);
}

.bot-name,
.member-name {
  font-size: 13px;
  font-weight: 700;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.bot-meta,
.member-meta {
  font-size: 12px;
  color: inherit;
  opacity: 0.9;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.empty-tip {
  margin: 0;
  padding: 10px;
  border: 1px dashed #d0d0d0;
  border-radius: 8px;
  font-size: 12px;
  color: #777;
}

.error-message {
  margin-bottom: 12px;
  padding: 9px 11px;
  border: 1px solid #e0b7b7;
  border-radius: 6px;
  background-color: #f3e7e7;
  color: #9b3c3c;
  font-size: 12px;
}

.section-card {
  background: #f5f5f5;
  border: 1px solid #d0d0d0;
  border-radius: 10px;
  padding: 12px;
}

@media (max-width: 768px) {
  .modal-content {
    max-height: 84vh;
  }

  .bot-list,
  .member-list {
    grid-template-columns: 1fr;
  }
}
</style>
