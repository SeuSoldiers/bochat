<template>
  <div class="page-shell home-shell">
    <TopNav />

    <div class="home-content">
      <div v-if="actionError" class="page-error">
        {{ actionError }}
      </div>

      <div class="page-tabs">
        <button
          :class="['page-tab', { active: activeTab === 'bots' }]"
          @click="activeTab = 'bots'"
        >
          Bot Console
        </button>
        <button
          :class="['page-tab', { active: activeTab === 'groups' }]"
          @click="activeTab = 'groups'"
        >
          Group Space
        </button>
      </div>

      <section v-if="activeTab === 'bots'" class="tab-content">
        <div class="section-header">
          <h2>My Bots</h2>
          <button class="btn btn-primary" @click="showCreateBotModal = true">
            + New Bot
          </button>
        </div>

        <div v-if="botStore.loading" class="loading">
          Loading...
        </div>

        <div v-else-if="botStore.bots.length > 0" class="card-grid">
          <BotCard
            v-for="bot in botStore.bots"
            :key="bot.bot_id"
            :bot="bot"
            @edit="openEditBot(bot)"
            @delete="handleDeleteBot(bot.bot_id)"
          />
        </div>

        <div v-else class="empty-state">
          <p>暂无 Bot</p>
          <p class="text-muted">点击上方按钮创建第一个 Bot</p>
        </div>
      </section>

      <section v-if="activeTab === 'groups'" class="tab-content">
        <div class="section-header">
          <div>
            <h2>My Groups</h2>
            <p class="section-tip">选择群后可在成员弹窗中添加或移除 Bot</p>
          </div>
          <div class="actions">
            <button class="btn btn-primary" @click="showCreateGroupModal = true">
              + New Group
            </button>
            <button class="btn btn-secondary" @click="showJoinGroupModal = true">
              Join Group
            </button>
          </div>
        </div>

        <div v-if="groupStore.loading" class="loading">
          Loading...
        </div>

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

        <div v-else class="empty-state">
          <p>暂无群聊</p>
          <p class="text-muted">创建新群或通过群号加入</p>
        </div>
      </section>
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
.home-content {
  background: rgba(255, 255, 255, 0.72);
  border: 1px solid #e5ebe5;
  border-radius: 22px;
  box-shadow: 0 16px 30px rgba(12, 36, 29, 0.1);
  padding: 22px;
}

.page-tabs {
  display: inline-flex;
  gap: 6px;
  padding: 6px;
  margin-bottom: 24px;
  background: #f8fbf6;
  border: 1px solid #dbe3db;
  border-radius: 999px;
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
  color: #64726c;
  padding: 10px 16px;
  border-radius: 999px;
  font-size: 14px;
  font-weight: 700;
  transition: var(--transition-base);
}

.page-tab.active {
  background-color: #0e3c2f;
  color: #f3f9f5;
  box-shadow: 0 10px 18px rgba(14, 60, 47, 0.24);
}

.section-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: 30px;
}

.section-header h2 {
  font-size: 26px;
  font-weight: 700;
  color: #102b23;
}

.section-tip {
  margin-top: 4px;
  font-size: 13px;
  color: #65756f;
}

.actions {
  display: flex;
  gap: 10px;
}

.btn {
  padding: 10px 16px;
  border: 1px solid transparent;
  border-radius: 999px;
  font-size: 13px;
  font-weight: 700;
  transition: var(--transition-base);
}

.btn-primary {
  background: #a6d72e;
  color: #173329;
  border-color: #9ccf2a;
}

.btn-primary:hover {
  background-color: #b5de46;
  transform: translateY(-1px);
}

.btn-secondary {
  background-color: #f5f7f3;
  color: #3f4d47;
  border-color: #d8dfd8;
}

.btn-secondary:hover {
  background-color: #edf3e8;
}

.tab-content {
  animation: fadeIn 0.24s ease;
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

  .home-content {
    padding: 16px;
    border-radius: 16px;
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
    gap: 12px;
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
