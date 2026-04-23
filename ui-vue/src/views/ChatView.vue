<template>
  <div class="page-shell chat-shell">
    <TopNav />

    <div class="chat-main">
      <div class="chat-header">
        <BotGroupSelector
          :groups="groupStore.groups"
          :selected-group="groupStore.selectedGroup"
          @select-group="groupStore.selectGroup"
        />

        <div v-if="groupStore.selectedGroup" class="group-summary">
          <div class="group-info">
            <h3>{{ groupStore.selectedGroup.name }}</h3>
            <p>Group Code: {{ groupStore.selectedGroup.group_code || 'Not Set' }}</p>
          </div>
          <button class="view-members-btn" @click="openMembersModal">
            Members
          </button>
        </div>
      </div>

      <div class="message-area">
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
      </div>

      <MessageInput
        v-if="groupStore.selectedGroup"
        :group-id="groupStore.selectedGroup.group_id"
        :bots="botStore.bots"
        :initial-bot-id="activeBotId"
        @send="handleSendMessage"
        @send-file="handleSendFile"
      />
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
    messageError.value = '请先选择一个 Bot'
    return
  }

  try {
    messageError.value = null
    activeBotId.value = botId
    const botToken = getBotToken(botId)
    if (!botToken) {
      throw new Error('未找到对应 Bot Token')
    }
    await chatStore.addMessage({
      group_id: groupStore.selectedGroup.group_id,
      content: { text: content },
      msg_type: 'text',
      idempotency_key: createIdempotencyKey(),
    }, botToken)
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
    messageError.value = '请先选择一个 Bot'
    return
  }

  try {
    messageError.value = null
    activeBotId.value = botId
    const botToken = getBotToken(botId)
    if (!botToken) {
      throw new Error('未找到对应 Bot Token')
    }

    const uploaded = await uploadFile(file, botToken)
    await chatStore.addMessage({
      group_id: groupStore.selectedGroup.group_id,
      content: {
        url: uploaded.url,
        filename: uploaded.filename || file.name,
      },
      msg_type: 'file',
      idempotency_key: createIdempotencyKey(),
    }, botToken)
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

const handleAddBotToGroup = async (botId: string) => {
  if (!groupStore.selectedGroup) {
    return
  }

  await groupStore.addBotToGroup(groupStore.selectedGroup.group_id, botId)
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
.chat-main {
  flex: 1;
  display: flex;
  flex-direction: column;
  padding: 20px;
  gap: 16px;
  min-height: 0;
  border-radius: 22px;
  border: 1px solid #e4eae3;
  background: rgba(255, 255, 255, 0.72);
  box-shadow: 0 16px 30px rgba(12, 36, 29, 0.1);
}

.chat-shell {
  display: flex;
  flex-direction: column;
}

.chat-header {
  display: flex;
  align-items: stretch;
  gap: 14px;
}

.group-summary {
  flex: 1;
  min-width: 0;
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 15px 18px;
  background: linear-gradient(110deg, #f8fbf6 0%, #eff4eb 100%);
  border-radius: 16px;
  border: 1px solid #d8e0d8;
}

.group-info h3 {
  font-size: 18px;
  font-weight: 700;
  color: #102d23;
  margin: 0 0 4px 0;
}

.group-info p {
  font-size: 12px;
  color: #61716b;
  margin: 0;
}

.view-members-btn {
  padding: 9px 14px;
  background: #0e3c2f;
  color: #ebf2ee;
  border: 1px solid #1f5d4a;
  border-radius: 999px;
  font-size: 13px;
  font-weight: 700;
  transition: var(--transition-base);
}

.view-members-btn:hover {
  transform: translateY(-1px);
  background: #165242;
}

.message-area {
  flex: 0 1 68%;
  display: flex;
  flex-direction: column;
  gap: 15px;
  min-height: 0;
  max-height: 68%;
}

.empty-chat {
  flex: 1;
  display: flex;
  align-items: center;
  justify-content: center;
  color: #8d9994;
  font-size: 16px;
  border-radius: 16px;
  border: 1px dashed #cfdbcf;
  background: rgba(247, 250, 245, 0.64);
  min-height: 300px;
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

  .chat-main {
    padding: 14px;
    border-radius: 16px;
  }

  .message-area {
    flex: 1;
    max-height: none;
  }

  .chat-header {
    flex-direction: column;
    align-items: stretch;
    gap: 10px;
  }

  .group-summary {
    flex-direction: column;
    align-items: flex-start;
    gap: 10px;
  }

  .view-members-btn {
    width: 100%;
  }
}
</style>
