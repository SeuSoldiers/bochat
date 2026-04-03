<template>
  <div class="home-container">
    <!-- 顶部导航 -->
    <TopNav />

    <!-- 主内容区 -->
    <div class="home-content">
      <div v-if="actionError" class="page-error">
        {{ actionError }}
      </div>

      <div class="page-tabs">
        <button
          :class="['page-tab', { active: activeTab === 'bots' }]"
          @click="activeTab = 'bots'"
        >
          Bot 管理
        </button>
        <button
          :class="['page-tab', { active: activeTab === 'groups' }]"
          @click="activeTab = 'groups'"
        >
          群聊管理
        </button>
      </div>

      <!-- Bot 管理标签页 -->
      <section v-if="activeTab === 'bots'" class="tab-content">
        <div class="section-header">
          <h2>我的 Bot</h2>
          <button class="btn btn-primary" @click="showCreateBotModal = true">
            + 创建 Bot
          </button>
        </div>

        <!-- 加载状态 -->
        <div v-if="botStore.loading" class="loading">
          加载中...
        </div>

        <!-- Bot 列表 -->
        <div v-else-if="botStore.bots.length > 0" class="card-grid">
          <BotCard
            v-for="bot in botStore.bots"
            :key="bot.bot_id"
            :bot="bot"
            @edit="openEditBot(bot)"
            @delete="handleDeleteBot(bot.bot_id)"
          />
        </div>

        <!-- 空状态 -->
        <div v-else class="empty-state">
          <p>还没有创建任何 Bot</p>
          <p class="text-muted">点击上面的按钮创建你的第一个 Bot</p>
        </div>
      </section>

      <!-- 群聊管理标签页 -->
      <section v-if="activeTab === 'groups'" class="tab-content">
        <div class="section-header">
          <div>
            <h2>我的群</h2>
            <p class="section-tip">先选群，再在弹窗里选择你的 Bot 来加群或退群</p>
          </div>
          <div class="actions">
            <button class="btn btn-primary" @click="showCreateGroupModal = true">
              + 创建群
            </button>
            <button class="btn btn-secondary" @click="showJoinGroupModal = true">
              加入群
            </button>
          </div>
        </div>

        <!-- 加载状态 -->
        <div v-if="groupStore.loading" class="loading">
          加载中...
        </div>

        <!-- 群列表 -->
        <div v-else-if="groupStore.groups.length > 0" class="card-grid">
          <GroupCard
            v-for="group in groupStore.groups"
            :key="group.group_id"
            :group="group"
            :selected="group.group_id === groupStore.selectedGroupId"
            :can-delete="group.creator_id === authStore.userId"
            @select="groupStore.selectGroup(group.group_id)"
            @delete="handleDeleteGroup(group.group_id)"
            @view-members="showGroupMembers(group.group_id)"
          />
        </div>

        <!-- 空状态 -->
        <div v-else class="empty-state">
          <p>还没有加入任何群</p>
          <p class="text-muted">创建新群或加入现有的群</p>
        </div>
      </section>
    </div>

    <!-- 创建 Bot 模态框 -->
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

    <!-- 创建群模态框 -->
    <CreateGroupModal
      v-if="showCreateGroupModal"
      :bots="botStore.bots"
      @create="handleCreateGroup"
      @close="showCreateGroupModal = false"
    />

    <!-- 加入群模态框 -->
    <JoinGroupModal
      v-if="showJoinGroupModal"
      :bots="botStore.bots"
      @join="handleJoinGroup"
      @close="showJoinGroupModal = false"
    />

    <!-- 群成员模态框 -->
    <MembersModal
      v-if="showMembersModal && selectedGroupForMembers"
      :group="selectedGroupForMembers"
      :members="groupStore.groupMembers[selectedGroupForMembers.group_id] || []"
      :owned-bots="botStore.bots"
      :loading="groupStore.loading"
      @add-bot="handleAddBotToGroup"
      @remove-bot="handleRemoveBotFromGroup"
      @close="showMembersModal = false"
    />
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted } from 'vue'
import { useBotStore } from '@/stores/bots'
import { useGroupStore } from '@/stores/groups'
import { useAuthStore } from '@/stores/auth'
import TopNav from '@/components/Common/TopNav.vue'
import BotCard from '@/components/Bot/BotCard.vue'
import EditBotModal from '@/components/Bot/EditBotModal.vue'
import GroupCard from '@/components/Group/GroupCard.vue'
import CreateBotModal from '@/components/Bot/CreateBotModal.vue'
import CreateGroupModal from '@/components/Group/CreateGroupModal.vue'
import JoinGroupModal from '@/components/Group/JoinGroupModal.vue'
import MembersModal from '@/components/Group/MembersModal.vue'
import type { Bot, Group } from '@/types'

const botStore = useBotStore()
const groupStore = useGroupStore()
const authStore = useAuthStore()

const activeTab = ref<'bots' | 'groups'>('bots')
const showCreateBotModal = ref(false)
const showEditBotModal = ref(false)
const showCreateGroupModal = ref(false)
const showJoinGroupModal = ref(false)
const showMembersModal = ref(false)
const editingBot = ref<Bot | null>(null)
const selectedGroupForMembers = ref<Group | null>(null)
const actionError = ref<string | null>(null)

// 初始化
onMounted(() => {
  groupStore.initializeSelectedGroup()
  botStore.fetchBots()
  groupStore.fetchGroups()
})

// 创建 Bot
const handleCreateBot = async (name: string, description: string, avatarUrl: string) => {
  try {
    actionError.value = null
    await botStore.addBot({ name, description, avatar_url: avatarUrl || undefined })
    showCreateBotModal.value = false
  } catch (error) {
    actionError.value = botStore.error || '创建 Bot 失败'
    console.error('Failed to create bot:', error)
  }
}

// 删除 Bot
const handleDeleteBot = async (botId: string) => {
  if (confirm('确定要删除这个 Bot 吗？')) {
    try {
      actionError.value = null
      await botStore.removeBotById(botId)
    } catch (error) {
      actionError.value = botStore.error || '删除 Bot 失败'
      console.error('Failed to delete bot:', error)
    }
  }
}

const openEditBot = (bot: Bot) => {
  editingBot.value = bot
  showEditBotModal.value = true
}

const handleEditBot = async (payload: { name: string; description: string; avatarUrl: string }) => {
  if (!editingBot.value) {
    return
  }

  try {
    actionError.value = null
    await botStore.updateBotInfo(editingBot.value.bot_id, {
      name: payload.name,
      description: payload.description || undefined,
      avatar_url: payload.avatarUrl || undefined,
    })
    showEditBotModal.value = false
    editingBot.value = null
  } catch (error) {
    actionError.value = botStore.error || '更新 Bot 失败'
    console.error('Failed to update bot:', error)
  }
}

// 创建群
const handleCreateGroup = async (groupName: string, groupNumber: string, botId: string) => {
  try {
    actionError.value = null
    await groupStore.addGroup({ name: groupName, group_code: groupNumber, bot_id: botId })
    showCreateGroupModal.value = false
  } catch (error) {
    actionError.value = groupStore.error || '创建群失败'
    console.error('Failed to create group:', error)
  }
}

// 删除群
const handleDeleteGroup = async (groupId: string) => {
  if (confirm('确定要删除这个群吗？')) {
    try {
      actionError.value = null
      await groupStore.removeGroupById(groupId)
    } catch (error) {
      actionError.value = groupStore.error || '删除群失败'
      console.error('Failed to delete group:', error)
    }
  }
}

// 加入群
const handleJoinGroup = async (groupNumber: string, botId: string) => {
  try {
    actionError.value = null
    await groupStore.joinGroupByNumber(groupNumber, botId)
    showJoinGroupModal.value = false
    await groupStore.fetchGroups()
  } catch (error) {
    actionError.value = groupStore.error || '加入群失败'
    console.error('Failed to join group:', error)
  }
}

// 查看群成员
const showGroupMembers = async (groupId: string) => {
  const group = groupStore.groups.find((g) => g.group_id === groupId)
  if (group) {
    selectedGroupForMembers.value = group
    await groupStore.fetchGroupMembers(groupId)
    showMembersModal.value = true
  }
}

const handleAddBotToGroup = async (botId: string) => {
  if (!selectedGroupForMembers.value) {
    return
  }

  try {
    actionError.value = null
    await groupStore.addBotToGroup(selectedGroupForMembers.value.group_id, botId)
  } catch (error) {
    actionError.value = groupStore.error || '添加 Bot 到群聊失败'
    console.error('Failed to add bot to group:', error)
  }
}

const handleRemoveBotFromGroup = async (botId: string) => {
  if (!selectedGroupForMembers.value) {
    return
  }

  try {
    actionError.value = null
    await groupStore.removeBotFromGroup(selectedGroupForMembers.value.group_id, botId)
  } catch (error) {
    actionError.value = groupStore.error || '移出群聊失败'
    console.error('Failed to remove bot from group:', error)
  }
}
</script>

<style scoped>
.home-container {
  display: flex;
  flex-direction: column;
  min-height: 100vh;
}

.home-content {
  flex: 1;
  padding: 30px;
  background-color: #f5f3f1;
}

.page-tabs {
  display: inline-flex;
  gap: 8px;
  padding: 6px;
  margin-bottom: 24px;
  background: white;
  border: 1px solid #d4cfc8;
  border-radius: 10px;
}

.page-error {
  margin-bottom: 16px;
  padding: 12px 14px;
  border: 1px solid #e0b4aa;
  border-radius: 8px;
  background: #fbf0ed;
  color: #9e5647;
  font-size: 13px;
}

.page-tab {
  border: none;
  background: transparent;
  color: #888888;
  padding: 10px 16px;
  border-radius: 8px;
  font-size: 14px;
  font-weight: 600;
  cursor: pointer;
  transition: all 0.2s ease;
}

.page-tab.active {
  background-color: #8b9d83;
  color: white;
}

.section-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: 30px;
}

.section-header h2 {
  font-size: 24px;
  font-weight: 600;
  color: #4a4a4a;
}

.section-tip {
  margin-top: 6px;
  font-size: 13px;
  color: #888888;
}

.actions {
  display: flex;
  gap: 10px;
}

.btn {
  padding: 10px 16px;
  border: none;
  border-radius: 6px;
  font-size: 13px;
  font-weight: 500;
  cursor: pointer;
  transition: all 0.3s ease;
}

.btn-primary {
  background-color: #8b9d83;
  color: white;
}

.btn-primary:hover {
  background-color: #9caa93;
}

.btn-secondary {
  background-color: #d4cfc8;
  color: #4a4a4a;
}

.btn-secondary:hover {
  background-color: #e8e3dd;
}

.tab-content {
  animation: fadeIn 0.3s ease;
}

.card-grid {
  display: grid;
  grid-template-columns: repeat(2, minmax(0, 1fr));
  gap: 24px;
}

.loading,
.empty-state {
  text-align: center;
  padding: 40px 20px;
  color: #888888;
}

.empty-state p {
  margin: 8px 0;
}

.empty-state .text-muted {
  font-size: 13px;
  color: #cccccc;
}

@keyframes fadeIn {
  from {
    opacity: 0;
  }
  to {
    opacity: 1;
  }
}

@media (max-width: 768px) {
  .home-content {
    padding: 20px;
  }

  .page-tabs {
    display: flex;
    width: 100%;
  }

  .page-tab {
    flex: 1;
  }

  .section-header {
    flex-direction: column;
    align-items: flex-start;
    gap: 15px;
  }

  .actions {
    width: 100%;
  }

  .btn {
    flex: 1;
  }

  .card-grid {
    grid-template-columns: 1fr;
  }
}
</style>
