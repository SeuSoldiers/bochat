<template>
  <div class="login-container">
    <div class="login-card">
      <h1 class="login-title">BoChat</h1>
      <p class="login-subtitle">智能聊天机器人管理平台</p>

      <form @submit.prevent="handleSubmit">
        <!-- 切换表单 -->
        <div class="form-tabs">
          <button
            type="button"
            :class="['tab', { active: isLogin }]"
            @click="isLogin = true"
          >
            登 录
          </button>
          <button
            type="button"
            :class="['tab', { active: !isLogin }]"
            @click="isLogin = false"
          >
            注 册
          </button>
        </div>

        <!-- 手机号输入 -->
        <div class="form-group">
          <label for="phone">手机号</label>
          <input
            id="phone"
            v-model="form.phone"
            type="tel"
            placeholder="请输入手机号"
            required
            :disabled="loading"
          />
        </div>

        <!-- 身份证号输入 -->
        <div class="form-group">
          <label for="idNumber">身份证号</label>
          <input
            id="idNumber"
            v-model="form.idNumber"
            type="text"
            placeholder="请输入身份证号"
            required
            :disabled="loading"
          />
        </div>

        <!-- 错误提示 -->
        <div v-if="error" class="error-message">
          {{ error }}
        </div>

        <!-- 提交按钮 -->
        <button type="submit" class="submit-btn" :disabled="loading">
          <span v-if="!loading">{{ isLogin ? '登 录' : '注 册' }}</span>
          <span v-else>处理中...</span>
        </button>
      </form>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref } from 'vue'
import { useRouter } from 'vue-router'
import { useAuthStore } from '@/stores/auth'

const router = useRouter()
const authStore = useAuthStore()

const isLogin = ref(true)
const loading = ref(false)
const form = ref({
  phone: '',
  idNumber: '',
})
const error = ref<string | null>(null)

const handleSubmit = async () => {
  error.value = null
  loading.value = true

  try {
    if (isLogin.value) {
      await authStore.handleLogin(form.value.phone, form.value.idNumber)
    } else {
      await authStore.handleRegister(form.value.phone, form.value.idNumber)
    }

    // 等待 router 导航完成
    await router.push('/home')

    // 导航成功后，清空表单
    form.value.phone = ''
    form.value.idNumber = ''
  } catch (err: any) {
    error.value = err.message || (isLogin.value ? '登录失败' : '注册失败')
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
  background: linear-gradient(135deg, #f5f3f1 0%, #ebe6e1 100%);
  padding: 20px;
}

.login-card {
  background: white;
  border-radius: 12px;
  padding: 40px;
  width: 100%;
  max-width: 400px;
  box-shadow: 0 4px 20px rgba(0, 0, 0, 0.1);
}

.login-title {
  font-size: 28px;
  font-weight: 600;
  color: #8b9d83;
  text-align: center;
  margin-bottom: 8px;
  letter-spacing: 1px;
}

.login-subtitle {
  font-size: 12px;
  color: #888888;
  text-align: center;
  margin-bottom: 30px;
}

.form-tabs {
  display: flex;
  gap: 0;
  margin-bottom: 30px;
  border-bottom: 1px solid #d4cfc8;
}

.tab {
  flex: 1;
  padding: 12px 0;
  border: none;
  background: none;
  color: #888888;
  font-size: 14px;
  font-weight: 500;
  cursor: pointer;
  position: relative;
  transition: all 0.3s ease;
}

.tab.active {
  color: #8b9d83;
}

.tab.active::after {
  content: '';
  position: absolute;
  bottom: -1px;
  left: 0;
  right: 0;
  height: 2px;
  background: #8b9d83;
}

.form-group {
  margin-bottom: 20px;
}

.form-group label {
  display: block;
  margin-bottom: 6px;
  font-size: 13px;
  color: #4a4a4a;
  font-weight: 500;
}

.form-group input {
  width: 100%;
  padding: 10px 12px;
  border: 1px solid #d4cfc8;
  border-radius: 6px;
  font-size: 14px;
  color: #4a4a4a;
  transition: all 0.3s ease;
}

.form-group input:focus {
  outline: none;
  border-color: #8b9d83;
  box-shadow: 0 0 0 3px rgba(139, 157, 131, 0.1);
}

.form-group input:disabled {
  background-color: #fafaf8;
  color: #cccccc;
  cursor: not-allowed;
}

.error-message {
  padding: 10px 12px;
  background-color: #f5e6e6;
  color: #a88b7f;
  border-radius: 6px;
  font-size: 13px;
  margin-bottom: 20px;
}

.submit-btn {
  width: 100%;
  padding: 12px;
  background-color: #8b9d83;
  color: white;
  border: none;
  border-radius: 6px;
  font-size: 14px;
  font-weight: 600;
  cursor: pointer;
  transition: all 0.3s ease;
}

.submit-btn:hover:not(:disabled) {
  background-color: #9caA93;
  box-shadow: 0 2px 8px rgba(139, 157, 131, 0.3);
}

.submit-btn:active:not(:disabled) {
  background-color: #7a8c72;
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
