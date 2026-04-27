<template>
  <div class="page-shell chat-shell">
    <div class="shell-body">
      <TopNav />

      <div class="chat-main">
        <aside class="chat-left">
          <div class="group-selector-wrap">
            <BotGroupSelector
              :groups="groupStore.groups"
              :selected-group="groupStore.selectedGroup"
              @select-group="groupStore.selectGroup"
            />
          </div>

          <div v-if="groupStore.selectedGroup" class="group-summary">
            <div class="group-avatar-wrap">
              <img
                v-if="groupStore.selectedGroup.avatar_url"
                :src="groupStore.selectedGroup.avatar_url"
                :alt="groupStore.selectedGroup.name"
                class="group-avatar"
              />
              <div v-else class="group-avatar fallback">{{ groupStore.selectedGroup.name.charAt(0) }}</div>
            </div>
            <div class="group-center">
              <h3>{{ groupStore.selectedGroup.name }}</h3>
              <p class="profile-desc">{{ groupStore.selectedGroup.description || '暂无群简介' }}</p>
            </div>
            <div class="group-right">
              <button class="view-members-btn" @click="openMembersModal">
                群成员
              </button>
              <p class="group-number">群号：{{ groupStore.selectedGroup.group_code || '未设置' }}</p>
            </div>
          </div>

          <div v-if="groupStore.selectedGroup" class="input-area">
            <MessageInput
              :group-id="groupStore.selectedGroup.group_id"
              :bots="botStore.bots"
              :initial-bot-id="activeBotId"
              @send="handleSendMessage"
              @send-file="handleSendFile"
            />
          </div>

          <div v-else class="left-empty-tip input-area">
            请选择群聊后发送消息
          </div>
        </aside>

        <section class="chat-right">
          <div v-if="messageError" class="page-error">
            {{ messageError }}
          </div>

          <MessageList
            v-if="groupStore.selectedGroup"
            :messages="chatStore.groupMessages"
            :loading="chatStore.loading"
            @view-bot="openBotInfo"
          />

          <div v-else class="empty-chat">
            <p>请选择群聊开始消息会话</p>
          </div>
        </section>
      </div>
    </div>

    <MembersModal
      v-if="showMembers && groupStore.selectedGroup"
      :group="groupStore.selectedGroup"
      :members="groupStore.selectedGroupMembers"
      :owned-bots="botStore.bots"
      :loading="groupStore.loading"
      @add-bot="handleAddBotToGroup"
      @remove-bot="handleRemoveBotFromGroup"
      @close="showMembers = false"
    />

    <BotInfoModal
      v-if="showBotInfoModal"
      :bot="viewingBot"
      :loading="loadingBotInfo"
      @close="showBotInfoModal = false"
    />
  </div>
</template>

<script setup lang="ts">
import { computed, ref, onMounted, watch } from 'vue'
import { useBotStore } from '@/stores/bots'
import { useGroupStore } from '@/stores/groups'
import { useChatStore } from '@/stores/chat'
import TopNav from '@/components/Common/TopNav.vue'
import BotGroupSelector from '@/components/Chat/BotGroupSelector.vue'
import MessageList from '@/components/Chat/MessageList.vue'
import MessageInput from '@/components/Chat/MessageInput.vue'
import MembersModal from '@/components/Group/MembersModal.vue'
import BotInfoModal from '@/components/Bot/BotInfoModal.vue'
import { getBot } from '@/services/bot'
import { uploadFile } from '@/services/file'
import type { Bot } from '@/types'
import { useWebSocket } from '@/composables/useWebSocket'
import { getErrorMessage } from '@/utils/error'

const botStore = useBotStore()
const groupStore = useGroupStore()
const chatStore = useChatStore()

const showMembers = ref(false)
const activeBotId = ref('')
const messageError = ref<string | null>(null)
const showBotInfoModal = ref(false)
const loadingBotInfo = ref(false)
const viewingBot = ref<Bot | null>(null)

const wsBotToken = computed(() => {
  const activeBot = botStore.bots.find((bot) => bot.bot_id === activeBotId.value)
  return activeBot?.token || botStore.bots[0]?.token || null
})

useWebSocket(wsBotToken)

// 初始化
onMounted(async () => {
  groupStore.initializeSelectedGroup()

  // 获取列表
  await botStore.fetchBots()
  await groupStore.fetchGroups()
  activeBotId.value = botStore.bots[0]?.bot_id || ''

  // 如果有选中的群，加载消息
  if (groupStore.selectedGroup) {
    chatStore.setCurrentGroup(groupStore.selectedGroup.group_id)
    await chatStore.fetchMessages(
      groupStore.selectedGroup.group_id,
      null,
      50,
      getBotToken(activeBotId.value)
    )
  }
})

// 监听群组和消息使用的 Bot 切换，自动加载消息
watch(
  () => [groupStore.selectedGroup?.group_id, activeBotId.value],
  async ([newGroupId, newBotId]) => {
    if (newGroupId) {
      chatStore.setCurrentGroup(newGroupId)
      await chatStore.fetchMessages(newGroupId, null, 50, getBotToken(newBotId))
    }
  }
)

// 发送消息
const handleSendMessage = async (content: string, botId: string) => {
  if (!groupStore.selectedGroup) {
    messageError.value = '请先选择一个群'
    return
  }

  if (!botId) {
    messageError.value = '请先选择一个机器人'
    return
  }

  try {
    messageError.value = null
    activeBotId.value = botId
    const botToken = getBotToken(botId)
    if (!botToken) {
      throw new Error('未找到对应机器人令牌')
    }
    const savedMessage = await chatStore.addMessage({
      group_id: groupStore.selectedGroup.group_id,
      content: { text: content },
      msg_type: 'text',
      idempotency_key: createIdempotencyKey(),
    }, botToken)
    // 立即更新页面，避免依赖 WS 回环
    chatStore.addWebSocketMessage(savedMessage)
  } catch (error: any) {
    messageError.value = getErrorMessage(error, '发送消息失败')
    console.error('Failed to send message:', error)
  }
}

const handleSendFile = async (file: File, botId: string) => {
  if (!groupStore.selectedGroup) {
    messageError.value = '请先选择一个群'
    return
  }

  if (!botId) {
    messageError.value = '请先选择一个机器人'
    return
  }

  try {
    messageError.value = null
    activeBotId.value = botId
    const botToken = getBotToken(botId)
    if (!botToken) {
      throw new Error('未找到对应机器人令牌')
    }

    const uploaded = await uploadFile(file, botToken)
    const savedMessage = await chatStore.addMessage({
      group_id: groupStore.selectedGroup.group_id,
      content: {
        url: uploaded.url,
        filename: uploaded.filename || file.name,
      },
      msg_type: 'file',
      idempotency_key: createIdempotencyKey(),
    }, botToken)
    // 立即更新页面，避免依赖 WS 回环
    chatStore.addWebSocketMessage(savedMessage)
  } catch (error: any) {
    messageError.value = getErrorMessage(error, '发送文件失败')
    console.error('Failed to send file:', error)
  }
}

const getBotToken = (botId?: string) => {
  if (!botId) {
    return ''
  }

  return botStore.bots.find((bot) => bot.bot_id === botId)?.token || ''
}

const createIdempotencyKey = () => {
  if (typeof crypto !== 'undefined' && typeof crypto.randomUUID === 'function') {
    return crypto.randomUUID()
  }

  return `msg-${Date.now()}-${Math.random().toString(16).slice(2)}`
}

const handleAddBotToGroup = async (payload: { botId: string; requestReason: string }) => {
  if (!groupStore.selectedGroup) {
    return
  }

  try {
    messageError.value = null
    const result = await groupStore.addBotToGroup(
      groupStore.selectedGroup.group_id,
      payload.botId,
      payload.requestReason
    )
    if (result.result_status === 'pending_approval') {
      messageError.value = result.message || '邀请已发送，等待对方同意'
    }
  } catch (error: any) {
    messageError.value = getErrorMessage(error, '邀请机器人失败')
  }
}

const handleRemoveBotFromGroup = async (botId: string) => {
  if (!groupStore.selectedGroup) {
    return
  }

  await groupStore.removeBotFromGroup(groupStore.selectedGroup.group_id, botId)
}

const openMembersModal = async () => {
  if (!groupStore.selectedGroup) {
    return
  }

  await groupStore.fetchGroupMembers(groupStore.selectedGroup.group_id)
  showMembers.value = true
}

const openBotInfo = async (botId: string) => {
  showBotInfoModal.value = true
  loadingBotInfo.value = true

  try {
    viewingBot.value = await getBot(botId)
  } catch (error) {
    console.error('Failed to load bot info:', error)
    viewingBot.value = null
  } finally {
    loadingBotInfo.value = false
  }
}
</script>

<style scoped>
.chat-shell {
  min-height: 0;
}

.shell-body {
  height: 100%;
  min-height: 0;
  display: flex;
  gap: 20px;
}

.chat-main {
  flex: 1;
  display: flex;
  flex-direction: row-reverse;
  gap: 14px;
  padding: 14px;
  min-height: 0;
  border-radius: 10px;
  border: 1px solid #d0d0d0;
  background: #f5f5f5;
  box-shadow: 0 4px 10px rgba(0, 0, 0, 0.05);
  overflow: hidden;
  align-items: stretch;
}

.chat-left {
  width: 360px;
  flex: 0 0 360px;
  min-width: 320px;
  min-height: 0;
  display: flex;
  flex-direction: column;
  gap: 10px;
}

.chat-right {
  flex: 1;
  min-width: 0;
  min-height: 0;
  display: flex;
  flex-direction: column;
  gap: 10px;
}

.input-area {
  flex: 1;
  min-height: 0;
  display: flex;
}

.input-area :deep(.message-input-container) {
  flex: 1;
  min-height: 0;
}

.input-area :deep(.message-input-form) {
  height: 100%;
  min-height: 0;
}

.input-area :deep(.input-wrapper) {
  flex: 1;
  min-height: 0;
}

.input-area :deep(.message-input) {
  height: 100%;
  min-height: 0;
  max-height: none;
  resize: none;
}

.group-selector-wrap {
  height: 148px;
  min-height: 148px;
  max-height: 148px;
  min-width: 0;
}

.group-selector-wrap :deep(.selector-container) {
  height: 100%;
  min-height: 0;
  flex: 0 0 auto;
  padding: 10px;
}

.group-selector-wrap :deep(.selector-section h3) {
  margin-bottom: 8px;
  font-size: 13px;
}

.group-selector-wrap :deep(.items-list) {
  max-height: 90px;
  overflow: auto;
  gap: 6px;
  padding-right: 2px;
}

.group-selector-wrap :deep(.item) {
  padding: 10px 14px;
  font-size: 13px;
  gap: 10px;
  max-width: 220px;
}

.group-summary {
  flex: 0 0 auto;
  min-width: 0;
  display: flex;
  justify-content: space-between;
  align-items: center;
  gap: 12px;
  padding: 15px 18px;
  background: #f5f5f5;
  border-radius: 8px;
  border: 1px solid #d0d0d0;
}

.group-center {
  min-width: 0;
  flex: 1;
}

.group-avatar-wrap {
  flex-shrink: 0;
}

.group-avatar {
  width: 68px;
  height: 68px;
  border-radius: 50%;
  object-fit: cover;
  background: #ebebeb;
}

.fallback {
  display: flex;
  align-items: center;
  justify-content: center;
  color: #2f2f2f;
  font-size: 28px;
  font-weight: 700;
}

.group-right {
  flex-shrink: 0;
  display: flex;
  flex-direction: column;
  align-items: flex-end;
  gap: 6px;
}

.profile-desc {
  margin: 0;
  color: #2f2f2f;
  font-size: 13px;
  line-height: 1.35;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.group-center h3 {
  font-size: 18px;
  font-weight: 700;
  color: #1f1f1f;
  margin: 0 0 4px 0;
}

.group-number {
  font-size: 12px;
  color: #5f5f5f;
  margin: 0;
}

.view-members-btn {
  padding: 9px 14px;
  background: #2f2f2f;
  color: #f3f3f3;
  border: 1px solid #2f2f2f;
  border-radius: 999px;
  font-size: 13px;
  font-weight: 700;
  transition: var(--transition-base);
}

.view-members-btn:hover {
  transform: translateY(-1px);
  background: #353535;
}

.empty-chat {
  flex: 1;
  display: flex;
  align-items: center;
  justify-content: center;
  color: #7f7f7f;
  font-size: 16px;
  border-radius: 8px;
  border: 1px dashed #d0d0d0;
  background: #f5f5f5;
  min-height: 300px;
}

.left-empty-tip {
  border: 1px dashed #d0d0d0;
  border-radius: 8px;
  background: #f5f5f5;
  color: #7f7f7f;
  font-size: 13px;
  min-height: 120px;
  display: grid;
  place-items: center;
}

.page-error {
  padding: 12px 16px;
  border: 1px solid #efc7c2;
  border-radius: 12px;
  background: #fff2ef;
  color: #a2443c;
  font-size: 13px;
}

@media (max-width: 768px) {
  .chat-shell {
    padding: 12px;
  }

  .shell-body {
    flex-direction: column;
    gap: 12px;
  }

  .chat-main {
    padding: 14px;
    border-radius: 8px;
    flex-direction: column;
  }

  .chat-left {
    width: 100%;
    min-width: 0;
    flex: 0 0 auto;
  }

  .group-selector-wrap {
    height: auto;
    min-height: 0;
    max-height: none;
  }

  .group-selector-wrap :deep(.items-list) {
    max-height: 160px;
  }

  .chat-right {
    min-height: 420px;
  }

  .group-summary {
    flex-direction: column;
    gap: 10px;
    align-items: flex-start;
  }

  .group-right {
    width: 100%;
    align-items: stretch;
  }

  .view-members-btn {
    width: 100%;
  }
}
</style>
