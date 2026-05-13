<template>
  <aside class="bot-side-nav">
    <div class="brand">
      <h1>BoChat</h1>
      <div class="brand-status" role="status" aria-live="polite">
        <span class="status-dot" :class="store.connectionStatus" aria-hidden="true"></span>
      </div>
    </div>

    <nav class="main-menu">
      <button
        v-for="item in menuItems"
        :key="item.key"
        type="button"
        class="menu-item"
        :class="{ active: store.activeView === item.key }"
        @click="store.setActiveView(item.key)"
      >
        <span class="item-icon" aria-hidden="true">
          <component :is="item.icon" class="icon-svg" />
        </span>
        <span class="menu-label">{{ item.label }}</span>
      </button>
    </nav>

    <div class="bot-card">
      <div class="bot-avatar">
        <Bot class="bot-avatar-icon" />
      </div>
      <div class="bot-meta">
        <p class="bot-name">{{ store.botIdentity.name }}</p>
      </div>

      <div class="bot-selector">
        <label class="bot-selector-label">发言 Bot</label>
        <select
          v-model="selectedBotId"
          class="bot-select"
          @change="handleBotChange"
        >
          <option
            v-for="bot in store.bots"
            :key="bot.bot_id"
            :value="bot.bot_id"
          >
            {{ bot.name }}
          </option>
          <option v-if="store.bots.length === 0" value="" disabled>
            暂无 Bot
          </option>
        </select>
      </div>

      <button type="button" class="manage-bot-btn" @click="goToBotManagement">
        <Settings class="icon-xs" />
        <span>管理 Bot</span>
      </button>

      <div class="bot-status">
        <span class="status-dot-small" :class="store.connectionStatus"></span>
        <span class="status-text">{{ store.connectionLabel }}</span>
      </div>
    </div>
  </aside>
</template>

<script setup lang="ts">
import { ref, watch } from 'vue'
import { useRouter } from 'vue-router'
import { MessageSquare, Users, FolderOpen, Activity, Bot, Settings } from 'lucide-vue-next'
import { useBotConsoleStore } from '@/stores/botConsole'

const store = useBotConsoleStore()
const router = useRouter()

const menuItems = [
  { key: 'chat' as const, label: '实时聊天', icon: MessageSquare },
  { key: 'groups' as const, label: '群组空间', icon: Users },
  { key: 'files' as const, label: '文件记录', icon: FolderOpen },
  { key: 'logs' as const, label: '连接日志', icon: Activity },
]

const selectedBotId = ref(store.activeBotId)

watch(() => store.activeBotId, (newId) => {
  selectedBotId.value = newId
})

const handleBotChange = () => {
  store.activeBotId = selectedBotId.value
}

const goToBotManagement = () => {
  router.push('/home')
}
</script>

<style scoped>
.bot-side-nav {
  position: relative;
  z-index: 5;
  width: 248px;
  flex: 0 0 248px;
  height: 100%;
  min-height: 0;
  border-radius: 10px;
  padding: 22px 18px;
  display: flex;
  flex-direction: column;
  gap: 20px;
  color: #1f1f1f;
  background: #f5f5f5;
  border: 1px solid #d0d0d0;
  box-shadow: 0 4px 10px rgba(0, 0, 0, 0.05);
}

.brand {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 8px;
  padding: 2px 8px 16px;
}

.brand h1 {
  margin: 0;
  font-size: 24px;
  line-height: 1;
  font-weight: 700;
  color: #1a1a1a;
}

.brand-status {
  display: inline-flex;
  align-items: center;
  margin-left: auto;
}

.status-dot {
  width: 10px;
  height: 10px;
  border-radius: 50%;
  background: #9f9f9f;
  box-shadow: 0 0 0 3px rgba(159, 159, 159, 0.2);
}

.status-dot.connected {
  background: #2f8f4e;
  box-shadow: 0 0 0 3px rgba(47, 143, 78, 0.2);
}

.status-dot.testing {
  background: #c9a227;
  box-shadow: 0 0 0 3px rgba(201, 162, 39, 0.2);
}

.status-dot.error,
.status-dot.disconnected {
  background: #c4453c;
  box-shadow: 0 0 0 3px rgba(196, 69, 60, 0.2);
}

.main-menu {
  display: flex;
  flex-direction: column;
  gap: 8px;
}

.menu-item {
  display: flex;
  align-items: center;
  gap: 10px;
  color: #2d2d2d;
  text-decoration: none;
  font-size: 14px;
  font-weight: 700;
  min-height: 46px;
  padding: 11px 10px;
  border-radius: 8px;
  border: 1px solid transparent;
  background: transparent;
  cursor: pointer;
  transition: var(--transition-base, all 0.2s ease);
  text-align: left;
}

.menu-label {
  min-width: 0;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.item-icon {
  width: 26px;
  height: 26px;
  display: grid;
  place-items: center;
  border-radius: 6px;
  color: #4a4a4a;
  background: transparent;
  flex-shrink: 0;
}

.icon-svg {
  width: 18px;
  height: 18px;
  stroke: currentColor;
  stroke-width: 2;
  stroke-linecap: round;
  stroke-linejoin: round;
  fill: none;
}

.menu-item:hover {
  border-color: #c8c8c8;
  background: #efefef;
}

.menu-item.active {
  color: #f3f3f3;
  border-color: #2f2f2f;
  background: #2f2f2f;
  box-shadow: none;
}

.menu-item.active .item-icon {
  color: #f3f3f3;
}

.bot-card {
  margin-top: auto;
  padding: 14px;
  border-radius: 8px;
  border: 1px solid #d0d0d0;
  background: #f5f5f5;
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 8px;
}

.bot-avatar {
  width: 48px;
  height: 48px;
  border-radius: 50%;
  background: #ebebeb;
  display: grid;
  place-items: center;
  border: 1px solid #d0d0d0;
}

.bot-avatar-icon {
  width: 24px;
  height: 24px;
  stroke: #4a4a4a;
  stroke-width: 2;
}

.bot-meta {
  text-align: center;
  min-width: 0;
  width: 100%;
}

.bot-name {
  margin: 0;
  font-size: 15px;
  font-weight: 700;
  color: #1f1f1f;
}

.bot-selector {
  width: 100%;
  display: flex;
  flex-direction: column;
  gap: 4px;
}

.bot-selector-label {
  font-size: 11px;
  font-weight: 600;
  color: #6f6f6f;
  text-transform: uppercase;
  letter-spacing: 0.5px;
}

.bot-select {
  width: 100%;
  padding: 6px 8px;
  border: 1px solid #d0d0d0;
  border-radius: 6px;
  background: #ffffff;
  color: #1f1f1f;
  font-size: 12px;
  font-weight: 600;
  font-family: inherit;
  cursor: pointer;
  outline: none;
  transition: border-color 0.2s ease;
}

.bot-select:focus {
  border-color: #7a7a7a;
  box-shadow: 0 0 0 3px rgba(120, 120, 120, 0.15);
}

.bot-select option {
  font-weight: 500;
  padding: 4px;
}

.manage-bot-btn {
  width: 100%;
  display: inline-flex;
  align-items: center;
  justify-content: center;
  gap: 6px;
  padding: 6px 12px;
  border-radius: 6px;
  border: 1px solid #d0d0d0;
  background: #ffffff;
  color: #4f4f4f;
  font-size: 12px;
  font-weight: 600;
  cursor: pointer;
  transition: all 0.2s ease;
}

.manage-bot-btn:hover {
  background: #ebebeb;
  border-color: #c5c5c5;
}

.bot-status {
  display: inline-flex;
  align-items: center;
  gap: 6px;
  font-size: 12px;
  color: #2f8f4e;
  font-weight: 600;
}

.status-dot-small {
  width: 8px;
  height: 8px;
  border-radius: 50%;
  background: #9f9f9f;
}

.status-dot-small.connected {
  background: #2f8f4e;
}

.status-dot-small.testing {
  background: #c9a227;
}

.status-dot-small.error,
.status-dot-small.disconnected {
  background: #c4453c;
}

@media (max-width: 1200px) {
  .bot-side-nav {
    width: 208px;
    flex-basis: 208px;
  }

  .brand h1 {
    font-size: 22px;
  }
}

@media (max-width: 768px) {
  .bot-side-nav {
    width: 100%;
    flex-basis: auto;
    padding: 14px;
    gap: 12px;
    border-radius: 8px;
    height: auto;
  }

  .main-menu {
    flex-direction: row;
    flex-wrap: wrap;
  }

  .menu-item {
    flex: 1;
    justify-content: center;
    font-size: 13px;
    padding: 10px;
    min-width: 80px;
  }

  .menu-label {
    display: none;
  }

  .bot-card {
    flex-direction: row;
    flex-wrap: wrap;
    justify-content: flex-start;
  }

  .bot-meta {
    text-align: left;
    flex: 1;
  }

  .bot-selector {
    flex: 1;
    min-width: 140px;
  }
}
</style>
