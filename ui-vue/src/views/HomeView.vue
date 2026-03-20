<template>
  <div class="home-container">
    <!-- 顶部导航 -->
    <TopNav />

    <!-- 主内容区 -->
    <div class="home-content">
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
            :selected="bot.bot_id === botStore.selectedBotId"
            @select="botStore.selectBot(bot.bot_id)"
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
          <h2>我的群</h2>
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
      @create="handleCreateBot"
      @close="showCreateBotModal = false"
    />

    <!-- 创建群模态框 -->
    <CreateGroupModal
      v-if="showCreateGroupModal"
      @create="handleCreateGroup"
      @close="showCreateGroupModal = false"
    />

    <!-- 加入群模态框 -->
    <JoinGroupModal
      v-if="showJoinGroupModal"
      @join="handleJoinGroup"
      @close="showJoinGroupModal = false"
    />

    <!-- 群成员模态框 -->
    <MembersModal
      v-if="showMembersModal && selectedGroupForMembers"
      :group="selectedGroupForMembers"
      :members="groupStore.groupMembers[selectedGroupForMembers.group_id] || []"
      @close="showMembersModal = false"
    />
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted } from 'vue'
import { useBotStore } from '@/stores/bots'
import { useGroupStore } from '@/stores/groups'
import TopNav from '@/components/Common/TopNav.vue'
import BotCard from '@/components/Bot/BotCard.vue'
import GroupCard from '@/components/Group/GroupCard.vue'
import CreateBotModal from '@/components/Bot/CreateBotModal.vue'
import CreateGroupModal from '@/components/Group/CreateGroupModal.vue'
import JoinGroupModal from '@/components/Group/JoinGroupModal.vue'
import MembersModal from '@/components/Group/MembersModal.vue'

const botStore = useBotStore()
const groupStore = useGroupStore()

const activeTab = ref<'bots' | 'groups'>('bots')
const showCreateBotModal = ref(false)
const showCreateGroupModal = ref(false)
const showJoinGroupModal = ref(false)
const showMembersModal = ref(false)
const selectedGroupForMembers = ref<any>(null)

// 初始化
onMounted(() => {
  botStore.initializeSelectedBot()
  groupStore.initializeSelectedGroup()
  botStore.fetchBots()
  groupStore.fetchGroups()
})

// 创建 Bot
const handleCreateBot = async (name: string, description: string) => {
  try {
    await botStore.addBot({ name, description })
    showCreateBotModal.value = false
  } catch (error) {
    console.error('Failed to create bot:', error)
  }
}

// 删除 Bot
const handleDeleteBot = async (botId: string) => {
  if (confirm('确定要删除这个 Bot 吗？')) {
    try {
      await botStore.removeBotById(botId)
    } catch (error) {
      console.error('Failed to delete bot:', error)
    }
  }
}

// 创建群
const handleCreateGroup = async (groupName: string, groupNumber: string) => {
  try {
    await groupStore.addGroup({ group_name: groupName, group_number: groupNumber })
    showCreateGroupModal.value = false
  } catch (error) {
    console.error('Failed to create group:', error)
  }
}

// 删除群
const handleDeleteGroup = async (groupId: string) => {
  if (confirm('确定要删除这个群吗？')) {
    try {
      await groupStore.removeGroupById(groupId)
    } catch (error) {
      console.error('Failed to delete group:', error)
    }
  }
}

// 加入群
const handleJoinGroup = async (groupNumber: string) => {
  try {
    await groupStore.joinGroupByNumber(groupNumber)
    showJoinGroupModal.value = false
    await groupStore.fetchGroups()
  } catch (error) {
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
  grid-template-columns: repeat(auto-fill, minmax(280px, 1fr));
  gap: 20px;
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
