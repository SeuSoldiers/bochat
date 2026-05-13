<template>
  <div class="login-container">
    <div class="login-card">
      <p class="eyebrow">智能运营中心</p>
      <h1 class="login-title">BoChat</h1>
      <p class="login-subtitle">智能聊天机器人管理平台</p>

      <form @submit.prevent="handleSubmit">
        <div class="form-tabs">
          <button
            type="button"
            :class="['tab', { active: isLogin }]"
            @click="switchMode(true)"
          >
            登录
          </button>
          <button
            type="button"
            :class="['tab', { active: !isLogin }]"
            @click="switchMode(false)"
          >
            注册
          </button>
        </div>

        <div v-if="!isLogin" class="form-group">
          <label for="name">昵称</label>
          <input
            id="name"
            v-model="form.name"
            type="text"
            autocomplete="nickname"
            placeholder="可选，不填则自动生成默认昵称"
            :disabled="loading"
          />
        </div>

        <div class="form-group">
          <label for="account">账号</label>
          <input
            id="account"
            v-model="form.account"
            type="text"
            autocomplete="username"
            placeholder="4-32位，仅支持字母、数字、下划线"
            :disabled="loading"
          />
        </div>

        <div class="form-group">
          <label for="password">密码</label>
          <input
            id="password"
            v-model="form.password"
            type="password"
            autocomplete="current-password"
            placeholder="8-64位，需包含字母和数字"
            :disabled="loading"
          />
        </div>

        <div v-if="error" class="error-message">
          {{ error }}
        </div>

        <button type="submit" class="submit-btn" :disabled="loading">
          <span v-if="!loading">{{ isLogin ? '进入控制台' : '创建并进入' }}</span>
          <span v-else>处理中...</span>
        </button>
      </form>
    </div>
  </div>
</template>

<script setup lang="ts">
import { computed, ref } from 'vue'
import { useRoute, useRouter } from 'vue-router'
import { useAuthStore } from '@/stores/auth'
import { getErrorMessage } from '@/utils/error'

const router = useRouter()
const route = useRoute()
const authStore = useAuthStore()

const isLogin = ref(true)
const loading = ref(false)
const form = ref({
  name: '',
  account: '',
  password: '',
})
const error = ref<string | null>(null)

const redirectPath = computed(() => {
  const redirect = route.query.redirect
  if (typeof redirect === 'string' && redirect.startsWith('/') && redirect !== '/login') {
    return redirect
  }
  return '/'
})

function validateCredentials(account: string, password: string): string | null {
  const accountPattern = /^[A-Za-z0-9_]{4,32}$/
  const passwordPattern = /^(?=.*[A-Za-z])(?=.*\d)[\x21-\x7E]{8,64}$/

  if (!accountPattern.test(account)) {
    return '账号格式不正确：长度需为4-32位，仅支持字母、数字、下划线'
  }

  if (!passwordPattern.test(password)) {
    return '密码格式不正确：长度需为8-64位，且必须包含字母和数字'
  }

  return null
}

const switchMode = (loginMode: boolean) => {
  isLogin.value = loginMode
  error.value = null
}

const handleSubmit = async () => {
  error.value = null
  loading.value = true

  try {
    const account = form.value.account.trim()
    const password = form.value.password.trim()

    if (!account || !password) {
      error.value = '请输入账号和密码'
      return
    }

    const validationError = validateCredentials(account, password)
    if (validationError) {
      error.value = validationError
      return
    }

    const payload = { account, password }

    if (isLogin.value) {
      await authStore.handleLogin(payload)
    } else {
      await authStore.handleRegister({
        name: form.value.name.trim() || undefined,
        ...payload,
      })
    }

    form.value.name = ''
    form.value.account = ''
    form.value.password = ''

    await router.replace(redirectPath.value)
  } catch (err: any) {
    error.value = getErrorMessage(err, isLogin.value ? '登录失败' : '注册失败')
    console.error('登录/注册错误:', err)
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

.form-tabs {
  display: flex;
  gap: 0;
  margin-bottom: 30px;
  border-bottom: 1px solid #dbe2db;
}

.tab {
  flex: 1;
  padding: 12px 0;
  border: none;
  background: none;
  color: #6c7a75;
  font-size: 14px;
  font-weight: 700;
  cursor: pointer;
  position: relative;
  transition: var(--transition-base, all 0.2s ease);
}

.tab.active {
  color: #173329;
}

.tab.active::after {
  content: '';
  position: absolute;
  bottom: -1px;
  left: 0;
  right: 0;
  height: 2px;
  background: #222222;
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

.form-group input {
  width: 100%;
  padding: 10px 12px;
  border: 1px solid #d7dfd7;
  border-radius: 12px;
  font-size: 14px;
  color: #203129;
  transition: var(--transition-base, all 0.2s ease);
  background: #fbfdf9;
}

.form-group input:focus {
  outline: none;
  border-color: #7a7a7a;
  box-shadow: 0 0 0 4px rgba(120, 120, 120, 0.2);
}

.form-group input:disabled {
  background-color: #fafaf8;
  color: #cccccc;
  cursor: not-allowed;
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

.submit-btn:active:not(:disabled) {
  transform: translateY(0);
}

.submit-btn:disabled {
  opacity: 0.6;
  cursor: not-allowed;
}

@media (max-width: 480px) {
  .login-card {
    padding: 30px 20px;
  }

  .login-title {
    font-size: 24px;
  }
}
</style>
