<template>
  <div class="login-container">
    <div class="login-card">
      <p class="eyebrow">BOT CHAT ACCESS</p>
      <h1 class="login-title">BoChat</h1>
      <p class="login-subtitle">使用 Bot Token 进入实时聊天</p>

      <form @submit.prevent="handleSubmit">
        <div class="form-group">
          <label for="bot-token">Bot Token</label>
          <textarea
            id="bot-token"
            v-model="botToken"
            class="token-input"
            autocomplete="off"
            spellcheck="false"
            placeholder="粘贴 b_xxx:timestamp:signature 格式的 Bot Token"
            :disabled="loading"
          />
        </div>

        <div v-if="error" class="error-message">
          {{ error }}
        </div>

        <button type="submit" class="submit-btn" :disabled="loading">
          {{ loading ? '验证中...' : '进入聊天' }}
        </button>

        <router-link class="admin-link" to="/login">管理员账号登录</router-link>
      </form>
    </div>
  </div>
</template>

<script setup lang="ts">
import { computed, ref } from 'vue'
import { useRoute, useRouter } from 'vue-router'
import { useBotUserAuthStore } from '@/stores/botUserAuth'
import { getErrorMessage } from '@/utils/error'

const router = useRouter()
const route = useRoute()
const authStore = useBotUserAuthStore()

const botToken = ref('')
const loading = ref(false)
const error = ref<string | null>(null)

const redirectPath = computed(() => {
  const redirect = route.query.redirect
  if (typeof redirect === 'string' && redirect.startsWith('/bot/') && redirect !== '/bot/login') {
    return redirect
  }
  return '/bot/chat'
})

const handleSubmit = async () => {
  error.value = null
  const token = botToken.value.trim()

  if (!token) {
    error.value = '请输入 Bot Token'
    return
  }

  if (token.split(':').length !== 3) {
    error.value = 'Bot Token 格式不正确'
    return
  }

  loading.value = true
  try {
    await authStore.handleLogin(token)
    botToken.value = ''
    await router.replace(redirectPath.value)
  } catch (err: any) {
    error.value = getErrorMessage(err, 'Bot Token 登录失败')
  } finally {
    loading.value = false
  }
}
</script>

<style scoped>
.login-container {
  display: flex;
  align-items: center;
  justify-content: center;
  min-height: 100vh;
  background:
    radial-gradient(circle at 12% 18%, rgba(30, 30, 30, 0.2) 0, rgba(30, 30, 30, 0) 36%),
    radial-gradient(circle at 88% 12%, rgba(110, 110, 110, 0.2) 0, rgba(110, 110, 110, 0) 34%),
    linear-gradient(160deg, #ececec, #e4e4e4 100%);
  padding: 20px;
}

.login-card {
  background: rgba(255, 255, 255, 0.92);
  border: 1px solid #e0e8df;
  border-radius: 24px;
  padding: 40px;
  width: 100%;
  max-width: 430px;
  box-shadow: 0 26px 56px rgba(8, 29, 22, 0.2);
  backdrop-filter: blur(10px);
}

.eyebrow {
  margin: 0 0 8px;
  font-size: 11px;
  font-weight: 800;
  letter-spacing: 0.22em;
  color: #5c6d67;
}

.login-title {
  font-size: 34px;
  font-weight: 700;
  color: #0f2f25;
  text-align: left;
  margin: 0 0 8px;
}

.login-subtitle {
  font-size: 13px;
  color: #5d6b66;
  text-align: left;
  margin: 0 0 28px;
}

.form-group {
  margin-bottom: 20px;
}

.form-group label {
  display: block;
  margin-bottom: 6px;
  font-size: 13px;
  color: #24352e;
  font-weight: 700;
}

.token-input {
  width: 100%;
  min-height: 106px;
  padding: 10px 12px;
  border: 1px solid #d7dfd7;
  border-radius: 12px;
  font-size: 13px;
  color: #203129;
  transition: var(--transition-base, all 0.2s ease);
  background: #fbfdf9;
  resize: vertical;
  font-family: ui-monospace, SFMono-Regular, Consolas, monospace;
}

.token-input:focus {
  outline: none;
  border-color: #7a7a7a;
  box-shadow: 0 0 0 4px rgba(120, 120, 120, 0.2);
}

.error-message {
  padding: 10px 12px;
  background-color: #fff0ee;
  color: #a6453e;
  border-radius: 10px;
  font-size: 13px;
  margin-bottom: 20px;
  border: 1px solid #efc5bf;
}

.submit-btn {
  width: 100%;
  padding: 12px;
  background: linear-gradient(120deg, #222222 0%, #3b3b3b 100%);
  color: #f2f2f2;
  border: 1px solid #333333;
  border-radius: 12px;
  font-size: 14px;
  font-weight: 800;
  cursor: pointer;
  transition: var(--transition-base, all 0.2s ease);
}

.submit-btn:hover:not(:disabled) {
  transform: translateY(-1px);
  box-shadow: 0 14px 26px rgba(0, 0, 0, 0.28);
}

.submit-btn:disabled {
  opacity: 0.6;
  cursor: not-allowed;
}

.admin-link {
  display: block;
  margin-top: 16px;
  text-align: center;
  color: #4f4f4f;
  font-size: 13px;
  font-weight: 700;
  text-decoration: none;
}

.admin-link:hover {
  color: #1f1f1f;
}
</style>
