<template>
  <div class="page-shell home-shell">
    <div class="shell-body">
      <TopNav />

      <div class="home-content">
        <div v-if="actionError" class="page-error">
          {{ actionError }}
        </div>

        <div class="page-topbar">
          <div class="page-tabs">
            <button
              :class="['page-tab', { active: activeTab === 'bots' }]"
              @click="activeTab = 'bots'"
            >
              机器人控制台
            </button>
            <button
              :class="['page-tab', { active: activeTab === 'groups' }]"
              @click="activeTab = 'groups'"
            >
              群组空间
            </button>
          </div>

          <div class="topbar-actions">
            <button
              v-if="activeTab === 'bots'"
              class="btn btn-primary"
              @click="showCreateBotModal = true"
            >
              + 新建机器人
            </button>
            <div v-else class="actions">
              <button class="btn btn-primary" @click="showCreateGroupModal = true">
                + 新建群组
              </button>
              <button class="btn btn-secondary" @click="showJoinGroupModal = true">
                加入群组
              </button>
            </div>
          </div>
        </div>

        <section v-if="activeTab === 'bots'" class="tab-content">
          <div class="stats-panel">
            <div class="stat-item">
              <div class="stat-icon icon-bot">
                <svg viewBox="0 0 24 24" fill="none" aria-hidden="true">
                  <rect x="5" y="7" width="14" height="12" rx="3" />
                  <path d="M12 4v3M9 12h.01M15 12h.01M9 16h6" />
                </svg>
              </div>
              <div class="stat-meta">
                <p>机器人总数</p>
                <strong>{{ botStore.bots.length }}</strong>
              </div>
            </div>
            <div class="stat-divider"></div>
            <div class="stat-item">
              <div class="stat-icon icon-running">
                <svg viewBox="0 0 24 24" fill="none" aria-hidden="true">
                  <circle cx="12" cy="12" r="9" />
                  <path d="m8.7 12.3 2.2 2.2 4.4-4.4" />
                </svg>
              </div>
              <div class="stat-meta">
                <p>运行中</p>
                <strong>{{ runningBotsCount }}</strong>
              </div>
            </div>
            <div class="stat-divider"></div>
            <div class="stat-item">
              <div class="stat-icon icon-disabled">
                <svg viewBox="0 0 24 24" fill="none" aria-hidden="true">
                  <circle cx="12" cy="12" r="9" />
                  <path d="M10.2 8v8M13.8 8v8" />
                </svg>
              </div>
              <div class="stat-meta">
                <p>已停用</p>
                <strong>{{ disabledBotsCount }}</strong>
              </div>
            </div>
          </div>

          <div v-if="botStore.loading" class="loading">
            加载中...
          </div>

          <div v-else-if="botStore.bots.length > 0" class="card-list">
            <BotCard
              v-for="bot in botStore.bots"
              :key="bot.bot_id"
              :bot="bot"
              @edit="openEditBot(bot)"
              @delete="handleDeleteBot(bot.bot_id)"
            />
          </div>

          <div v-else class="empty-state">
            <p>暂无机器人</p>
            <p class="text-muted">点击上方按钮创建第一个机器人</p>
          </div>
        </section>

        <section v-if="activeTab === 'groups'" class="tab-content">
          <div v-if="groupStore.loading" class="loading">
            加载中...
          </div>

          <div v-else-if="groupStore.groups.length > 0" class="card-grid">
            <GroupCard
              v-for="group in groupStore.groups"
              :key="group.group_id"
              :group="group"
              :can-delete="group.creator_id === authStore.userId"
              @delete="handleDeleteGroup(group.group_id)"
              @view-members="showGroupMembers(group.group_id)"
            />
          </div>

          <div v-else class="empty-state">
            <p>暂无群聊</p>
            <p class="text-muted">创建新群或通过群号加入</p>
          </div>
        </section>
      </div>
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

    <CreateGroupModal
      v-if="showCreateGroupModal"
      :bots="botStore.bots"
      @create="handleCreateGroup"
      @close="showCreateGroupModal = false"
    />

    <JoinGroupModal
      v-if="showJoinGroupModal"
      :bots="botStore.bots"
      @join="handleJoinGroup"
      @close="showJoinGroupModal = false"
    />

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
import { computed, ref, onMounted } from 'vue'
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
const disabledStatuses = new Set(['disabled', 'stopped', 'paused', 'inactive'])

const disabledBotsCount = computed(
  () => botStore.bots.filter((bot) => disabledStatuses.has(bot.status?.toLowerCase())).length
)
const runningBotsCount = computed(() => Math.max(0, botStore.bots.length - disabledBotsCount.value))

// 初始化
onMounted(() => {
  groupStore.initializeSelectedGroup()
  botStore.fetchBots()
  groupStore.fetchGroups()
})

// 创建机器人
const handleCreateBot = async (name: string, description: string, avatarUrl: string) => {
  try {
    actionError.value = null
    await botStore.addBot({ name, description, avatar_url: avatarUrl || undefined })
    showCreateBotModal.value = false
  } catch (error) {
    actionError.value = botStore.error || '创建机器人失败'
    console.error('Failed to create bot:', error)
  }
}

// 删除机器人
const handleDeleteBot = async (botId: string) => {
  if (confirm('确定要删除这个机器人吗？')) {
    try {
      actionError.value = null
      await botStore.removeBotById(botId)
    } catch (error) {
      actionError.value = botStore.error || '删除机器人失败'
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
    actionError.value = botStore.error || '更新机器人失败'
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
    actionError.value = groupStore.error || '添加机器人到群聊失败'
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
.home-shell {
  min-height: 0;
}

.shell-body {
  height: 100%;
  min-height: 0;
  display: flex;
  gap: 20px;
}

.home-content {
  flex: 1;
  min-width: 0;
  min-height: 0;
  overflow: auto;
  background: #f5f5f5;
  border: 1px solid #d0d0d0;
  border-radius: 10px;
  box-shadow: 0 4px 10px rgba(0, 0, 0, 0.05);
  padding: 28px 30px;
}

.page-topbar {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 14px;
  margin-bottom: 18px;
}


.page-tabs {
  display: inline-flex;
  gap: 0;
  padding: 0;
  margin-bottom: 24px;
  background: #f5f5f5;
  border: 1px solid #d0d0d0;
  border-radius: 8px;
}

.topbar-actions {
  display: flex;
  align-items: center;
  justify-content: flex-end;
}

.page-topbar .page-tabs {
  margin-bottom: 0;
}

.page-error {
  margin-bottom: 16px;
  padding: 12px 16px;
  border: 1px solid #efc7c2;
  border-radius: 12px;
  background: #fff2ef;
  color: #a2443c;
  font-size: 13px;
}

.page-tab {
  border: none;
  background: transparent;
  color: #2f2f2f;
  padding: 12px 28px;
  border-radius: 6px;
  font-size: 16px;
  font-weight: 700;
  line-height: 1;
  transition: var(--transition-base);
}

.page-tab.active {
  background-color: #2f2f2f;
  color: #f3f3f3;
  box-shadow: none;
}

.section-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: 20px;
}

.section-header h2 {
  font-size: 46px;
  font-weight: 700;
  color: #1a1a1a;
  line-height: 1.15;
  margin-bottom: 12px;
}

.stats-panel {
  display: flex;
  align-items: center;
  border: 1px solid #d0d0d0;
  border-radius: 8px;
  background: #f5f5f5;
  padding: 8px 10px;
  margin-bottom: 18px;
}

.stat-item {
  flex: 1;
  min-width: 0;
  display: flex;
  align-items: center;
  gap: 10px;
  padding: 4px 10px;
}

.stat-divider {
  width: 1px;
  background: #d0d0d0;
}

.stat-icon {
  width: 34px;
  height: 34px;
  border-radius: 10px;
  display: grid;
  place-items: center;
}

.stat-icon svg {
  width: 18px;
  height: 18px;
  stroke: #2f2f2f;
  stroke-width: 1.8;
  stroke-linecap: round;
  stroke-linejoin: round;
}

.icon-bot {
  background: #ebebeb;
}

.icon-running {
  background: #ebebeb;
}

.icon-disabled {
  background: #ebebeb;
}

.icon-disabled svg {
  stroke: #7a7a7a;
}

.stat-meta p {
  margin: 0;
  font-size: 12px;
  color: #5d6661;
  line-height: 1.1;
}

.stat-meta strong {
  display: block;
  margin-top: 2px;
  font-size: 20px;
  color: #161616;
  line-height: 1;
}

.actions {
  display: flex;
  gap: 10px;
}

.btn {
  padding: 14px 26px;
  border: 1px solid transparent;
  border-radius: 8px;
  font-size: 16px;
  font-weight: 700;
  line-height: 1;
  transition: var(--transition-base);
}

.btn-primary {
  background: #242424;
  color: #f4f4f4;
  border-color: #303030;
}

.btn-primary:hover {
  background-color: #3a3a3a;
  transform: translateY(-1px);
}

.btn-secondary {
  background-color: #f5f5f5;
  color: #3f4d47;
  border-color: #d0d0d0;
}

.btn-secondary:hover {
  background-color: #ebebeb;
}

.tab-content {
  animation: fadeIn 0.24s ease;
}

.card-list {
  display: grid;
  grid-template-columns: repeat(2, minmax(0, 1fr));
  gap: 18px;
}

.card-grid {
  display: grid;
  grid-template-columns: repeat(2, minmax(0, 1fr));
  gap: 18px;
}

.loading,
.empty-state {
  text-align: center;
  padding: 44px 20px;
  color: #687670;
}

.empty-state p {
  margin: 8px 0;
}

.empty-state .text-muted {
  font-size: 13px;
  color: #98a39d;
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
  .home-shell {
    padding: 12px;
  }

  .shell-body {
    flex-direction: column;
    gap: 12px;
  }

  .home-content {
    padding: 16px;
    border-radius: 8px;
  }

  .page-topbar {
    flex-direction: column;
    align-items: stretch;
    gap: 12px;
  }

  .page-tabs {
    display: flex;
    width: 100%;
  }

  .page-tab {
    flex: 1;
    padding: 11px 12px;
    font-size: 14px;
  }

  .section-header {
    flex-direction: column;
    align-items: flex-start;
    gap: 12px;
  }

  .section-header h2 {
    font-size: 32px;
  }

  .stats-panel {
    flex-direction: column;
    gap: 10px;
  }

  .stat-divider {
    width: auto;
    height: 1px;
  }

  .stat-meta p {
    font-size: 14px;
  }

  .stat-meta strong {
    font-size: 24px;
  }

  .actions {
    width: 100%;
  }

  .card-list {
    grid-template-columns: 1fr;
    gap: 14px;
  }

  .card-grid {
    grid-template-columns: 1fr;
    gap: 14px;
  }

  .btn {
    flex: 1;
    font-size: 14px;
    border-radius: 8px;
  }
}
</style>
