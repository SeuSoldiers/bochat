<template>
  <div class="profile-page">
    <TopNav />

    <main class="profile-main">
      <section class="profile-card">
        <header class="card-header">
          <div>
            <h2>个人信息</h2>
            <p>管理你的昵称和密码</p>
          </div>
        </header>

        <div v-if="loadError" class="error-box">{{ loadError }}</div>

        <form class="profile-form" @submit.prevent="handleSave">
          <div class="form-group">
            <label for="name">昵称</label>
            <input id="name" v-model="form.name" type="text" placeholder="请输入昵称" :disabled="saving || loading" />
          </div>

          <div class="form-group">
            <label for="password">新密码</label>
            <input
              id="password"
              v-model="form.password"
              type="password"
              placeholder="留空表示不修改密码"
              :disabled="saving || loading"
            />
          </div>

          <div class="form-group">
            <label for="confirmPassword">确认新密码</label>
            <input
              id="confirmPassword"
              v-model="form.confirmPassword"
              type="password"
              placeholder="再次输入新密码"
              :disabled="saving || loading"
            />
          </div>

          <div v-if="saveError" class="error-box">{{ saveError }}</div>
          <div v-if="saveSuccess" class="success-box">{{ saveSuccess }}</div>

          <div class="actions">
            <button type="button" class="btn btn-secondary" :disabled="saving || loading" @click="resetForm">
              重置
            </button>
            <button type="submit" class="btn btn-primary" :disabled="saving || loading">
              {{ saving ? '保存中...' : '保存修改' }}
            </button>
          </div>
        </form>
      </section>
    </main>
  </div>
</template>

<script setup lang="ts">
import { onMounted, reactive, ref } from 'vue'
import TopNav from '@/components/Common/TopNav.vue'
import { useAuthStore } from '@/stores/auth'
import { getErrorMessage } from '@/utils/error'

const authStore = useAuthStore()

const loading = ref(false)
const saving = ref(false)
const loadError = ref<string | null>(null)
const saveError = ref<string | null>(null)
const saveSuccess = ref<string | null>(null)

const form = reactive({
  name: '',
  password: '',
  confirmPassword: '',
})

const fillForm = () => {
  form.name = authStore.user?.name || ''
  form.password = ''
  form.confirmPassword = ''
}

const fetchProfile = async () => {
  loading.value = true
  loadError.value = null

  try {
    await authStore.refreshCurrentUser()
    fillForm()
  } catch (err: any) {
    loadError.value = getErrorMessage(err, '加载用户信息失败')
  } finally {
    loading.value = false
  }
}

const resetForm = () => {
  saveError.value = null
  saveSuccess.value = null
  fillForm()
}

const handleSave = async () => {
  saveError.value = null
  saveSuccess.value = null
  saving.value = true

  try {
    const password = form.password.trim()
    const confirmPassword = form.confirmPassword.trim()

    if (password || confirmPassword) {
      if (password !== confirmPassword) {
        saveError.value = '两次输入的密码不一致'
        return
      }
    }

    await authStore.updateProfile({
      name: form.name.trim() || undefined,
      password: password || undefined,
    })
    saveSuccess.value = '保存成功'
    fillForm()
  } catch (err: any) {
    saveError.value = getErrorMessage(err, '保存失败')
  } finally {
    saving.value = false
  }
}

onMounted(fetchProfile)
</script>

<style scoped>
.profile-page {
  min-height: 100vh;
  background: linear-gradient(180deg, #f7f4f1 0%, #f0ece7 100%);
}

.profile-main {
  max-width: 880px;
  margin: 0 auto;
  padding: 28px 20px 40px;
}

.profile-card {
  background: #fff;
  border: 1px solid #e1dad3;
  border-radius: 14px;
  padding: 28px;
  box-shadow: 0 8px 24px rgba(78, 68, 59, 0.08);
}

.card-header h2 {
  margin: 0;
  font-size: 24px;
  color: #443831;
}

.card-header p {
  margin: 8px 0 0;
  color: #7d726a;
  font-size: 14px;
}

.profile-form {
  margin-top: 24px;
}

.form-group {
  margin-bottom: 18px;
}

.form-group label {
  display: block;
  margin-bottom: 6px;
  color: #594c44;
  font-size: 13px;
  font-weight: 600;
}

.form-group input {
  width: 100%;
  box-sizing: border-box;
  border: 1px solid #d7cec6;
  border-radius: 8px;
  padding: 10px 12px;
  font-size: 14px;
  color: #4c3e36;
  background: #fff;
}

.form-group input:focus {
  outline: none;
  border-color: #8b9d83;
  box-shadow: 0 0 0 3px rgba(139, 157, 131, 0.15);
}

.error-box,
.success-box {
  padding: 10px 12px;
  border-radius: 8px;
  font-size: 13px;
  margin-bottom: 12px;
}

.error-box {
  background: #fceeed;
  color: #9e4a45;
}

.success-box {
  background: #edf7ed;
  color: #2d7d46;
}

.actions {
  display: flex;
  justify-content: flex-end;
  gap: 10px;
}

.btn {
  border: none;
  border-radius: 8px;
  padding: 10px 16px;
  font-weight: 600;
  cursor: pointer;
}

.btn:disabled {
  opacity: 0.65;
  cursor: not-allowed;
}

.btn-primary {
  background: #8b9d83;
  color: #fff;
}

.btn-secondary {
  background: #ece6df;
  color: #6e5e52;
}

@media (max-width: 768px) {
  .profile-main {
    padding: 18px 12px 28px;
  }

  .profile-card {
    padding: 20px;
  }

  .actions {
    flex-direction: column;
  }

  .btn {
    width: 100%;
  }
}
</style>
