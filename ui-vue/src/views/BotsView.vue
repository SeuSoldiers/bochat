<template>
  <div class="bots-view">
    <header class="page-header">
      <div>
        <h1>Bot 管理</h1>
        <p class="page-sub">创建和管理你的聊天机器人</p>
      </div>
      <button type="button" class="btn-primary" @click="showCreateBotModal = true">
        <Plus class="icon-sm" />
        创建 Bot
      </button>
    </header>

    <div v-if="actionError" class="error-bar">{{ actionError }}</div>

    <div v-if="botStore.loading" class="loading">加载中...</div>

    <div v-else-if="botStore.bots.length > 0" class="bot-grid">
      <BotCard
        v-for="bot in botStore.bots"
        :key="bot.bot_id"
        :bot="bot"
        @edit="openEditBot(bot)"
        @join-group="openJoinGroup(bot)"
        @delete="handleDeleteBot(bot.bot_id)"
      />
    </div>

    <div v-else class="empty-state">
      <Bot class="empty-icon" />
      <p class="empty-title">还没有 Bot</p>
      <p class="empty-desc">Bot 是你在群聊中的发言人，创建一个来开始聊天</p>
      <button type="button" class="btn-primary" @click="showCreateBotModal = true">
        <Plus class="icon-sm" />
        创建第一个 Bot
      </button>
    </div>

    <CreateBotModal
      v-if="showCreateBotModal"
      :upload-token="botStore.bots[0]?.token"
      @create="handleCreateBot"
      @close="showCreateBotModal = false"
    />

    <EditBotModal
      v-if="showEditBotModal && editingBot"
      :bot="editingBot"
      @save="handleEditBot"
      @close="showEditBotModal = false"
    />

    <JoinGroupModal
      v-if="showJoinGroupModal && joiningBot"
      :groups="groupStore.groups"
      :preselected-bot-id="joiningBot?.bot_id"
      @join="handleJoinGroup"
      @close="closeJoinGroupModal"
    />
  </div>
</template>

<script setup lang="ts">
import { onMounted, ref } from 'vue'
import { Bot, Plus } from 'lucide-vue-next'
import { useBotStore } from '@/stores/bots'
import { useGroupStore } from '@/stores/groups'
import BotCard from '@/components/Bot/BotCard.vue'
import CreateBotModal from '@/components/Bot/CreateBotModal.vue'
import EditBotModal from '@/components/Bot/EditBotModal.vue'
import JoinGroupModal from '@/components/Group/JoinGroupModal.vue'
import type { Bot as BotType } from '@/types'

const botStore = useBotStore()
const groupStore = useGroupStore()

const actionError = ref<string | null>(null)
const showCreateBotModal = ref(false)
const showEditBotModal = ref(false)
const showJoinGroupModal = ref(false)
const editingBot = ref<BotType | null>(null)
const joiningBot = ref<BotType | null>(null)

const handleCreateBot = async (name: string, description: string, avatarUrl: string) => {
  try {
    actionError.value = null
    await botStore.addBot({ name, description, avatar_url: avatarUrl || undefined })
    showCreateBotModal.value = false
  } catch {
    actionError.value = botStore.error || '创建机器人失败'
  }
}

const handleDeleteBot = async (botId: string) => {
  if (confirm('确定要删除这个机器人吗？')) {
    try {
      actionError.value = null
      await botStore.removeBotById(botId)
    } catch {
      actionError.value = botStore.error || '删除机器人失败'
    }
  }
}

const openEditBot = (bot: BotType) => {
  editingBot.value = bot
  showEditBotModal.value = true
}

const openJoinGroup = (bot: BotType) => {
  joiningBot.value = bot
  showJoinGroupModal.value = true
}

const closeJoinGroupModal = () => {
  showJoinGroupModal.value = false
  joiningBot.value = null
}

const handleEditBot = async (payload: { name: string; description: string; avatarUrl: string }) => {
  if (!editingBot.value) return
  try {
    actionError.value = null
    await botStore.updateBotInfo(editingBot.value.bot_id, {
      name: payload.name,
      description: payload.description || undefined,
      avatar_url: payload.avatarUrl || undefined,
    })
    showEditBotModal.value = false
    editingBot.value = null
  } catch {
    actionError.value = botStore.error || '更新机器人失败'
  }
}

const handleJoinGroup = async (payload: {
  groupId?: string
  groupCode?: string
  botId: string
  requestReason: string
}) => {
  try {
    actionError.value = null
    if (payload.groupId) {
      await groupStore.joinGroupById(payload.groupId, payload.botId, payload.requestReason)
    } else if (payload.groupCode) {
      await groupStore.joinGroupByNumber(payload.groupCode, payload.botId, payload.requestReason)
    } else {
      throw new Error('未提供群标识')
    }
    closeJoinGroupModal()
    await groupStore.fetchGroups()
  } catch {
    actionError.value = groupStore.error || '加入群失败'
  }
}

onMounted(() => {
  botStore.fetchBots()
  groupStore.fetchGroups()
})
</script>

<style scoped>
.bots-view {
  width: 100%;
}

.page-header {
  display: flex;
  align-items: flex-start;
  justify-content: space-between;
  gap: 12px;
  margin-bottom: 20px;
}

.page-header h1 {
  margin: 0;
  font-size: 22px;
  font-weight: 700;
  color: #1a1a1a;
}

.page-sub {
  margin: 4px 0 0;
  font-size: 13px;
  color: #6f6f6f;
}

.btn-primary {
  display: inline-flex;
  align-items: center;
  gap: 6px;
  padding: 10px 18px;
  border: 1px solid #2f2f2f;
  border-radius: 8px;
  background: #2f2f2f;
  color: #f3f3f3;
  font-size: 13px;
  font-weight: 600;
  cursor: pointer;
  transition: all 0.15s ease;
  white-space: nowrap;
}

.btn-primary:hover {
  background: #3a3a3a;
}

.icon-sm {
  width: 16px;
  height: 16px;
  stroke: currentColor;
  stroke-width: 2;
}

.error-bar {
  padding: 10px 14px;
  margin-bottom: 16px;
  border: 1px solid #efc7c1;
  border-radius: 8px;
  background: #fff0ee;
  color: #a6453e;
  font-size: 13px;
}

.loading {
  text-align: center;
  padding: 48px;
  color: #6f6f6f;
  font-size: 14px;
}

.bot-grid {
  display: grid;
  grid-template-columns: repeat(2, 1fr);
  gap: 14px;
}

.empty-state {
  text-align: center;
  padding: 64px 20px;
  background: #fff;
  border: 1px solid #d0d0d0;
  border-radius: 8px;
}

.empty-icon {
  width: 48px;
  height: 48px;
  stroke: #bfbfbf;
  stroke-width: 1.5;
  margin-bottom: 12px;
}

.empty-title {
  margin: 0;
  font-size: 16px;
  font-weight: 700;
  color: #4f4f4f;
}

.empty-desc {
  margin: 8px 0 16px;
  font-size: 13px;
  color: #6f6f6f;
}

@media (max-width: 768px) {
  .bot-grid {
    grid-template-columns: 1fr;
  }

  .page-header {
    flex-direction: column;
  }
}
</style>
