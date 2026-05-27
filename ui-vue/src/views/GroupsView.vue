<template>
  <div class="groups-view">
    <header class="page-header">
      <div>
        <h1>群组管理</h1>
        <p class="page-sub">创建、加入和管理群组</p>
      </div>
      <div class="header-actions">
        <button type="button" class="btn-secondary" @click="showJoinGroupModal = true">
          <UserPlus class="icon-sm" />
          加入群组
        </button>
        <button type="button" class="btn-primary" @click="showCreateGroupModal = true">
          <Plus class="icon-sm" />
          创建群组
        </button>
      </div>
    </header>

    <div v-if="actionError" class="error-bar">{{ actionError }}</div>

    <div v-if="groupStore.loading" class="loading">加载中...</div>

    <div v-else-if="groupStore.groups.length > 0" class="group-grid">
      <GroupCard
        v-for="group in groupStore.groups"
        :key="group.group_id"
        :group="group"
        :can-edit="group.creator_id === authStore.userId"
        :can-delete="true"
        @edit="openEditGroup(group)"
        @delete="handleDeleteGroup(group.group_id)"
        @view-members="showGroupMembers(group.group_id)"
        @enter-chat="enterChat(group)"
      />
    </div>

    <div v-else class="empty-state">
      <Users class="empty-icon" />
      <p class="empty-title">还没有群组</p>
      <p class="empty-desc">群组是 Bot 聊天的地方，创建或加入一个来开始</p>
      <div class="empty-actions">
        <button type="button" class="btn-primary" @click="showCreateGroupModal = true">
          <Plus class="icon-sm" />
          创建群组
        </button>
        <button type="button" class="btn-secondary" @click="showJoinGroupModal = true">
          <UserPlus class="icon-sm" />
          加入群组
        </button>
      </div>
    </div>

    <CreateGroupModal
      v-if="showCreateGroupModal"
      :bots="botStore.bots"
      @create="handleCreateGroup"
      @close="showCreateGroupModal = false"
    />

    <JoinGroupModal
      v-if="showJoinGroupModal"
      :groups="groupStore.groups"
      :preselected-bot-id="botStore.bots[0]?.bot_id"
      @join="handleJoinGroup"
      @close="showJoinGroupModal = false"
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
import { onMounted, ref } from 'vue'
import { useRouter } from 'vue-router'
import { Users, Plus, UserPlus } from 'lucide-vue-next'
import { useBotStore } from '@/stores/bots'
import { useGroupStore } from '@/stores/groups'
import { useAuthStore } from '@/stores/auth'
import { useBotConsoleStore } from '@/stores/botConsole'
import GroupCard from '@/components/Group/GroupCard.vue'
import CreateGroupModal from '@/components/Group/CreateGroupModal.vue'
import EditGroupModal from '@/components/Group/EditGroupModal.vue'
import JoinGroupModal from '@/components/Group/JoinGroupModal.vue'
import MembersModal from '@/components/Group/MembersModal.vue'
import type { Group } from '@/types'

const router = useRouter()
const authStore = useAuthStore()
const botStore = useBotStore()
const groupStore = useGroupStore()
const botConsoleStore = useBotConsoleStore()

const actionError = ref<string | null>(null)
const showCreateGroupModal = ref(false)
const showJoinGroupModal = ref(false)
const showEditGroupModal = ref(false)
const showMembersModal = ref(false)
const editingGroup = ref<Group | null>(null)
const selectedGroupForMembers = ref<Group | null>(null)

const handleCreateGroup = async (
  groupName: string, description: string, groupNumber: string,
  botId: string, avatarUrl: string, isPublic: boolean,
) => {
  try {
    actionError.value = null
    await groupStore.addGroup({
      name: groupName, description: description || undefined,
      group_code: groupNumber, bot_id: botId,
      avatar_url: avatarUrl || undefined, is_public: isPublic,
    })
    showCreateGroupModal.value = false
  } catch {
    actionError.value = groupStore.error || '创建群失败'
  }
}

const handleDeleteGroup = async (groupId: string) => {
  if (confirm('确定要删除这个群吗？')) {
    try {
      actionError.value = null
      await groupStore.removeGroupById(groupId)
    } catch {
      actionError.value = groupStore.error || '删除群失败'
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
  name: string; description: string; groupCode: string; avatarUrl: string
}) => {
  if (!editingGroup.value) return
  try {
    actionError.value = null
    await groupStore.updateGroupInfo(editingGroup.value.group_id, {
      name: payload.name, description: payload.description || undefined,
      group_code: payload.groupCode || undefined, avatar_url: payload.avatarUrl || undefined,
    })
    closeEditGroupModal()
  } catch {
    actionError.value = groupStore.error || '更新群失败'
  }
}

const handleJoinGroup = async (payload: {
  groupId?: string; groupCode?: string; botId: string; requestReason: string
}) => {
  try {
    actionError.value = null
    const result = payload.groupId
      ? await groupStore.joinGroupById(payload.groupId, payload.botId, payload.requestReason)
      : await groupStore.joinGroupByNumber(payload.groupCode!, payload.botId, payload.requestReason)
    showJoinGroupModal.value = false
    if (result?.result_status === 'pending_approval') {
      actionError.value = result.message || '申请已提交，等待对方同意'
    } else {
      await groupStore.fetchGroups()
      actionError.value = null
    }
  } catch {
    actionError.value = groupStore.error || '加入群失败'
  }
}

const showGroupMembers = async (groupId: string) => {
  const group = groupStore.groups.find((g) => g.group_id === groupId)
  if (group) {
    selectedGroupForMembers.value = group
    await groupStore.fetchGroupMembers(groupId)
    showMembersModal.value = true
  }
}

const handleAddBotToGroup = async (payload: { botId: string; requestReason: string }) => {
  if (!selectedGroupForMembers.value) return
  try {
    actionError.value = null
    const result = await groupStore.addBotToGroup(
      selectedGroupForMembers.value.group_id, payload.botId, payload.requestReason,
    )
    if (result.result_status === 'pending_approval') {
      actionError.value = result.message || '邀请已发送，等待对方同意'
    }
  } catch {
    actionError.value = groupStore.error || '添加机器人到群聊失败'
  }
}

const handleRemoveBotFromGroup = async (botId: string) => {
  if (!selectedGroupForMembers.value) return
  try {
    actionError.value = null
    await groupStore.removeBotFromGroup(selectedGroupForMembers.value.group_id, botId)
  } catch {
    actionError.value = groupStore.error || '移出群聊失败'
  }
}

const enterChat = (group: Group) => {
  botConsoleStore.selectGroup(group.group_id)
  router.push('/chat')
}

onMounted(() => {
  groupStore.initializeSelectedGroup()
  botStore.fetchBots()
  groupStore.fetchGroups()
})
</script>

<style scoped>
.groups-view {
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

.header-actions {
  display: flex;
  gap: 8px;
}

.btn-primary,
.btn-secondary {
  display: inline-flex;
  align-items: center;
  gap: 6px;
  padding: 10px 18px;
  border-radius: 8px;
  font-size: 13px;
  font-weight: 600;
  cursor: pointer;
  transition: all 0.15s ease;
  white-space: nowrap;
}

.btn-primary {
  border: 1px solid #2f2f2f;
  background: #2f2f2f;
  color: #f3f3f3;
}

.btn-primary:hover {
  background: #3a3a3a;
}

.btn-secondary {
  border: 1px solid #d0d0d0;
  background: #f5f5f5;
  color: #4f4f4f;
}

.btn-secondary:hover {
  background: #ebebeb;
  border-color: #c5c5c5;
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

.group-grid {
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

.empty-actions {
  display: flex;
  gap: 10px;
  justify-content: center;
}

@media (max-width: 768px) {
  .group-grid {
    grid-template-columns: 1fr;
  }

  .page-header {
    flex-direction: column;
  }
}
</style>
