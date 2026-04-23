<template>
  <div class="page-shell profile-page">
    <TopNav />

    <main class="profile-main">
      <section class="profile-card">
        <header class="card-header">
          <div>
            <h2>Profile Settings</h2>
            <p>管理昵称、密码与账户资料</p>
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
  min-height: calc(100vh - 40px);
}

.profile-main {
  max-width: 920px;
  margin: 0 auto 8px;
  padding: 8px 8px 22px;
}

.profile-card {
  background: rgba(255, 255, 255, 0.86);
  border: 1px solid #dde5dd;
  border-radius: 22px;
  padding: 30px;
  box-shadow: 0 18px 34px rgba(9, 30, 22, 0.14);
}

.card-header h2 {
  margin: 0;
  font-size: 28px;
  color: #113026;
}

.card-header p {
  margin: 8px 0 0;
  color: #64726d;
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
  color: #2f4039;
  font-size: 13px;
  font-weight: 700;
}

.form-group input {
  width: 100%;
  box-sizing: border-box;
  border: 1px solid #d6dfd6;
  border-radius: 12px;
  padding: 10px 12px;
  font-size: 14px;
  color: #22332b;
  background: #fafdfa;
}

.form-group input:focus {
  outline: none;
  border-color: #99cb27;
  box-shadow: 0 0 0 4px rgba(166, 215, 46, 0.2);
}

.error-box,
.success-box {
  padding: 10px 12px;
  border-radius: 8px;
  font-size: 13px;
  margin-bottom: 12px;
}

.error-box {
  background: #fff0ee;
  color: #a6453e;
  border: 1px solid #efc7c1;
}

.success-box {
  background: #ecf9d7;
  color: #29513f;
  border: 1px solid #cde895;
}

.actions {
  display: flex;
  justify-content: flex-end;
  gap: 10px;
}

.btn {
  border: 1px solid transparent;
  border-radius: 999px;
  padding: 10px 16px;
  font-weight: 700;
  transition: var(--transition-base);
}

.btn:disabled {
  opacity: 0.65;
  cursor: not-allowed;
}

.btn-primary {
  background: #a6d72e;
  color: #123126;
  border-color: #98c52c;
}

.btn-secondary {
  background: #f5f7f3;
  color: #50605a;
  border-color: #d8dfd8;
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
