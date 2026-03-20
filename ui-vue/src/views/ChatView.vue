<template>
  <div class="chat-container">
    <!-- 顶部导航 -->
    <TopNav />

    <!-- 主聊天区 -->
    <div class="chat-main">
      <!-- Bot 和群选择器 -->
      <BotGroupSelector
        :bots="botStore.bots"
        :groups="groupStore.groups"
        :selected-bot="botStore.selectedBot"
        :selected-group="groupStore.selectedGroup"
        @select-bot="botStore.selectBot"
        @select-group="groupStore.selectGroup"
      />

      <!-- 消息区域 -->
      <div class="message-area">
        <!-- 头部：当前群信息 -->
        <div v-if="groupStore.selectedGroup" class="chat-header">
          <div class="group-info">
            <h3>{{ groupStore.selectedGroup.group_name }}</h3>
            <p>群号: {{ groupStore.selectedGroup.group_number }}</p>
          </div>
          <button class="view-members-btn" @click="showMembers = true">
            👥 成员 ({{ groupStore.selectedGroup.member_count }})
          </button>
        </div>

        <!-- 消息列表 -->
        <MessageList
          v-if="groupStore.selectedGroup"
          :messages="chatStore.groupMessages"
          :loading="chatStore.loading"
        />

        <!-- 未选择群 -->
        <div v-else class="empty-chat">
          <p>请选择一个群开始聊天</p>
        </div>
      </div>

      <!-- 输入区域 -->
      <MessageInput
        v-if="groupStore.selectedGroup && botStore.selectedBot"
        :group-id="groupStore.selectedGroup.group_id"
        :bot-id="botStore.selectedBot.bot_id"
        @send="handleSendMessage"
      />
    </div>

    <!-- 成员列表模态框 -->
    <MembersModal
      v-if="showMembers && groupStore.selectedGroup"
      :group="groupStore.selectedGroup"
      :members="groupStore.selectedGroupMembers"
      @close="showMembers = false"
    />
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted, watch } from 'vue'
import { useBotStore } from '@/stores/bots'
import { useGroupStore } from '@/stores/groups'
import { useChatStore } from '@/stores/chat'
import { useAuthStore } from '@/stores/auth'
import { useWebSocket } from '@/composables/useWebSocket'
import TopNav from '@/components/Common/TopNav.vue'
import BotGroupSelector from '@/components/Chat/BotGroupSelector.vue'
import MessageList from '@/components/Chat/MessageList.vue'
import MessageInput from '@/components/Chat/MessageInput.vue'
import MembersModal from '@/components/Group/MembersModal.vue'

const botStore = useBotStore()
const groupStore = useGroupStore()
const chatStore = useChatStore()
const authStore = useAuthStore()

const showMembers = ref(false)
const messageError = ref<string | null>(null)

// WebSocket 连接
const { } = useWebSocket(authStore.token)

// 初始化
onMounted(async () => {
  // 初始化选中的 Bot 和群
  botStore.initializeSelectedBot()
  groupStore.initializeSelectedGroup()

  // 获取列表
  await botStore.fetchBots()
  await groupStore.fetchGroups()

  // 如果有选中的群，加载消息
  if (groupStore.selectedGroup) {
    chatStore.setCurrentGroup(groupStore.selectedGroup.group_id)
    await chatStore.fetchMessages(groupStore.selectedGroup.group_id)
  }
})

// 监听群组切换，自动加载消息
watch(
  () => groupStore.selectedGroup?.group_id,
  async (newGroupId) => {
    if (newGroupId) {
      chatStore.setCurrentGroup(newGroupId)
      await chatStore.fetchMessages(newGroupId)
    }
  }
)

// 发送消息
const handleSendMessage = async (content: string) => {
  if (!groupStore.selectedGroup) {
    messageError.value = '请先选择一个群'
    return
  }

  try {
    messageError.value = null
    await chatStore.addMessage({
      group_id: groupStore.selectedGroup.group_id,
      content,
      message_type: 'text',
    })
  } catch (error: any) {
    messageError.value = error.message || '发送消息失败'
    console.error('Failed to send message:', error)
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
  gap: 20px;
  max-width: 1200px;
  margin: 0 auto;
  width: 100%;
}

.chat-header {
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
  margin-bottom: 4px;
}

.group-info p {
  font-size: 12px;
  color: #888888;
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

@media (max-width: 768px) {
  .chat-main {
    padding: 15px;
  }

  .chat-header {
    flex-direction: column;
    align-items: flex-start;
    gap: 10px;
  }

  .view-members-btn {
    width: 100%;
  }
}
</style>
