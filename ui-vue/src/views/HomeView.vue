<template>
  <div class="page-shell home-shell">
    <div class="shell-body">
      <TopNav />

      <div class="home-content">
        <div v-if="actionError" class="page-error">
          {{ actionError }}
        </div>

        <JoinRequestPanel
          :inbox="groupStore.joinRequestsInbox"
          :outbox="groupStore.joinRequestsOutbox"
          :loading="groupStore.loading"
          @approve="handleApproveJoinRequest"
          @reject="handleRejectJoinRequest"
        />

        <section v-if="activeTab === 'bots'" class="tab-content">
          <div v-if="botStore.loading" class="loading">
            加载中...
          </div>

          <div v-else-if="botStore.bots.length > 0" class="card-list">
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
              :can-edit="group.creator_id === authStore.userId"
              :can-delete="group.creator_id === authStore.userId"
              @edit="openEditGroup(group)"
              @delete="handleDeleteGroup(group.group_id)"
              @view-members="showGroupMembers(group.group_id)"
            />
          </div>

          <div v-else class="empty-state">
            <p>暂无群聊</p>
            <p class="text-muted">创建新群或通过群号加入</p>
          </div>
        </section>

        <button
          class="fab-create-btn"
          :aria-label="activeTab === 'bots' ? '新建机器人' : '新建群聊'"
          @click="activeTab === 'bots' ? (showCreateBotModal = true) : (showCreateGroupModal = true)"
        >
          <span class="fab-plus" aria-hidden="true">+</span>
        </button>
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
      v-if="showJoinGroupModal && joiningBot"
      :groups="groupStore.groups"
      :preselected-bot-id="joiningBot?.bot_id"
      @join="handleJoinGroup"
      @close="closeJoinGroupModal"
    />

    <EditGroupModal
      v-if="showEditGroupModal && editingGroup"
      :group="editingGroup"
      :upload-token="botStore.bots[0]?.token"
      @save="handleEditGroup"
      @close="closeEditGroupModal"
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
import { useRoute } from 'vue-router'
import { useBotStore } from '@/stores/bots'
import { useGroupStore } from '@/stores/groups'
import { useAuthStore } from '@/stores/auth'
import TopNav from '@/components/Common/TopNav.vue'
import BotCard from '@/components/Bot/BotCard.vue'
import EditBotModal from '@/components/Bot/EditBotModal.vue'
import GroupCard from '@/components/Group/GroupCard.vue'
import CreateBotModal from '@/components/Bot/CreateBotModal.vue'
import CreateGroupModal from '@/components/Group/CreateGroupModal.vue'
import EditGroupModal from '@/components/Group/EditGroupModal.vue'
import JoinGroupModal from '@/components/Group/JoinGroupModal.vue'
import MembersModal from '@/components/Group/MembersModal.vue'
import JoinRequestPanel from '@/components/Group/JoinRequestPanel.vue'
import type { Bot, Group } from '@/types'

const botStore = useBotStore()
const groupStore = useGroupStore()
const authStore = useAuthStore()
const route = useRoute()

const activeTab = computed<'bots' | 'groups'>(() =>
  route.query.tab === 'groups' ? 'groups' : 'bots'
)
const showCreateBotModal = ref(false)
const showEditBotModal = ref(false)
const showCreateGroupModal = ref(false)
const showEditGroupModal = ref(false)
const showJoinGroupModal = ref(false)
const showMembersModal = ref(false)
const editingBot = ref<Bot | null>(null)
const editingGroup = ref<Group | null>(null)
const joiningBot = ref<Bot | null>(null)
const selectedGroupForMembers = ref<Group | null>(null)
const actionError = ref<string | null>(null)

// 初始化
onMounted(() => {
  groupStore.initializeSelectedGroup()
  botStore.fetchBots()
  groupStore.fetchGroups()
  groupStore.fetchJoinRequests('inbox')
  groupStore.fetchJoinRequests('outbox')
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

const openJoinGroup = (bot: Bot) => {
  joiningBot.value = bot
  showJoinGroupModal.value = true
}

const closeJoinGroupModal = () => {
  showJoinGroupModal.value = false
  joiningBot.value = null
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
const handleCreateGroup = async (
  groupName: string,
  description: string,
  groupNumber: string,
  botId: string,
  avatarUrl: string
) => {
  try {
    actionError.value = null
    await groupStore.addGroup({
      name: groupName,
      description: description || undefined,
      group_code: groupNumber,
      bot_id: botId,
      avatar_url: avatarUrl || undefined,
    })
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

const openEditGroup = (group: Group) => {
  editingGroup.value = group
  showEditGroupModal.value = true
}

const closeEditGroupModal = () => {
  showEditGroupModal.value = false
  editingGroup.value = null
}

const handleEditGroup = async (payload: {
  name: string
  description: string
  groupCode: string
  avatarUrl: string
}) => {
  if (!editingGroup.value) {
    return
  }

  try {
    actionError.value = null
    await groupStore.updateGroupInfo(editingGroup.value.group_id, {
      name: payload.name,
      description: payload.description || undefined,
      group_code: payload.groupCode || undefined,
      avatar_url: payload.avatarUrl || undefined,
    })
    closeEditGroupModal()
  } catch (error) {
    actionError.value = groupStore.error || '更新群失败'
    console.error('Failed to update group:', error)
  }
}

// 加入群
const handleJoinGroup = async (payload: {
  groupId?: string
  groupCode?: string
  botId: string
  requestReason: string
}) => {
  try {
    actionError.value = null
    let result: { result_status?: string; message?: string } | null = null
    if (payload.groupId) {
      result = await groupStore.joinGroupById(payload.groupId, payload.botId, payload.requestReason)
    } else if (payload.groupCode) {
      result = await groupStore.joinGroupByNumber(payload.groupCode, payload.botId, payload.requestReason)
    } else {
      throw new Error('未提供群标识')
    }
    closeJoinGroupModal()
    if (result?.result_status === 'pending_approval') {
      actionError.value = result.message || '申请已提交，等待对方同意'
    } else {
      await groupStore.fetchGroups()
      actionError.value = null
    }
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

const handleAddBotToGroup = async (payload: { botId: string; requestReason: string }) => {
  if (!selectedGroupForMembers.value) {
    return
  }

  try {
    actionError.value = null
    const result = await groupStore.addBotToGroup(
      selectedGroupForMembers.value.group_id,
      payload.botId,
      payload.requestReason
    )
    if (result.result_status === 'pending_approval') {
      actionError.value = result.message || '邀请已发送，等待对方同意'
    }
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

const handleApproveJoinRequest = async (requestId: string) => {
  try {
    actionError.value = null
    await groupStore.approveJoinRequestById(requestId)
    if (selectedGroupForMembers.value) {
      await groupStore.fetchGroupMembers(selectedGroupForMembers.value.group_id)
    }
  } catch (error) {
    actionError.value = groupStore.error || '同意申请失败'
    console.error('Failed to approve join request:', error)
  }
}

const handleRejectJoinRequest = async (requestId: string) => {
  try {
    actionError.value = null
    await groupStore.rejectJoinRequestById(requestId)
  } catch (error) {
    actionError.value = groupStore.error || '拒绝申请失败'
    console.error('Failed to reject join request:', error)
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
  position: relative;
  background: #f5f5f5;
  border: 1px solid #d0d0d0;
  border-radius: 10px;
  box-shadow: 0 4px 10px rgba(0, 0, 0, 0.05);
  padding: 28px 30px;
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

.fab-create-btn {
  position: absolute;
  right: 30px;
  bottom: 30px;
  width: 52px;
  height: 52px;
  border-radius: 999px;
  border: 1px solid #2f2f2f;
  background: #2f2f2f;
  color: #f3f3f3;
  display: grid;
  place-items: center;
  cursor: pointer;
  box-shadow: 0 6px 16px rgba(0, 0, 0, 0.22);
  transition: var(--transition-base);
}

.fab-plus {
  font-size: 32px;
  line-height: 1;
  font-weight: 500;
  transform: translateY(-1px);
}

.fab-create-btn:hover {
  background: #3a3a3a;
  transform: translateY(-1px);
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

  .fab-create-btn {
    right: 20px;
    bottom: 20px;
    width: 48px;
    height: 48px;
  }

  .fab-plus {
    font-size: 30px;
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
