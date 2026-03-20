<template>
  <div class="chat-container">
    <!-- 顶部导航 -->
    <TopNav />

    <!-- 主聊天区 -->
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
            <p>群号: {{ groupStore.selectedGroup.group_code || '未设置' }}</p>
          </div>
          <button class="view-members-btn" @click="openMembersModal">
            👥 成员
          </button>
        </div>
      </div>

      <!-- 消息区域 -->
      <div class="message-area">
        <div v-if="messageError" class="page-error">
          {{ messageError }}
        </div>

        <!-- 消息列表 -->
        <MessageList
          v-if="groupStore.selectedGroup"
          :messages="chatStore.groupMessages"
          :loading="chatStore.loading"
          @view-bot="openBotInfo"
        />

        <!-- 未选择群 -->
        <div v-else class="empty-chat">
          <p>请选择一个群开始聊天</p>
        </div>
      </div>

      <!-- 输入区域 -->
      <MessageInput
        v-if="groupStore.selectedGroup"
        :group-id="groupStore.selectedGroup.group_id"
        :bots="botStore.bots"
        :initial-bot-id="activeBotId"
        @send="handleSendMessage"
      />
    </div>

    <!-- 成员列表模态框 -->
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
      activeBotId.value,
      50,
      0,
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
      await chatStore.fetchMessages(newGroupId, newBotId, 50, 0, getBotToken(newBotId))
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
      bot_id: botId,
    }, botToken)
  } catch (error: any) {
    messageError.value = getErrorMessage(error, '发送消息失败')
    console.error('Failed to send message:', error)
  }
}

const getBotToken = (botId?: string) => {
  if (!botId) {
    return ''
  }

  return botStore.bots.find((bot) => bot.bot_id === botId)?.token || ''
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
.chat-container {
  display: flex;
  flex-direction: column;
  min-height: 100vh;
  background-color: #f5f3f1;
}

.chat-main {
  flex: 1;
  display: flex;
  flex-direction: column;
  padding: 20px;
  gap: 16px;
  max-width: 1200px;
  margin: 0 auto;
  width: 100%;
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
  padding: 15px;
  background-color: white;
  border-radius: 8px;
  border: 1px solid #d4cfc8;
}

.group-info h3 {
  font-size: 16px;
  font-weight: 600;
  color: #4a4a4a;
  margin: 0 0 4px 0;
}

.group-info p {
  font-size: 12px;
  color: #888888;
  margin: 0;
}

.view-members-btn {
  padding: 8px 12px;
  background-color: #8b9d83;
  color: white;
  border: none;
  border-radius: 6px;
  font-size: 13px;
  cursor: pointer;
  transition: all 0.3s ease;
}

.view-members-btn:hover {
  background-color: #9caa93;
}

.message-area {
  flex: 1;
  display: flex;
  flex-direction: column;
  gap: 15px;
  min-height: 400px;
}

.empty-chat {
  flex: 1;
  display: flex;
  align-items: center;
  justify-content: center;
  color: #cccccc;
  font-size: 16px;
}

.page-error {
  padding: 12px 14px;
  border: 1px solid #e0b4aa;
  border-radius: 8px;
  background: #fbf0ed;
  color: #9e5647;
  font-size: 13px;
}

@media (max-width: 768px) {
  .chat-main {
    padding: 15px;
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
