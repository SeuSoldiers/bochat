<template>
  <div class="profile-page">
    <TopNav />

    <main class="profile-main">
      <section class="profile-card">
        <header class="card-header">
          <div>
            <h2>个人信息</h2>
            <p>管理你的用户名、手机号和头像</p>
          </div>
        </header>

        <div v-if="loadError" class="error-box">{{ loadError }}</div>

        <form class="profile-form" @submit.prevent="handleSave">
          <div class="form-group">
            <label for="name">用户名</label>
            <input id="name" v-model="form.name" type="text" placeholder="请输入用户名" :disabled="saving || loading" />
          </div>

          <div class="form-group">
            <label for="phone">手机号</label>
            <input
              id="phone"
              v-model="form.phone"
              type="tel"
              placeholder="留空表示不使用手机号登录"
              :disabled="saving || loading"
            />
          </div>

          <div class="form-group">
            <label for="avatarUrl">头像 URL</label>
            <input
              id="avatarUrl"
              v-model="form.avatarUrl"
              type="url"
              placeholder="https://example.com/avatar.png"
              :disabled="saving || loading"
            />
          </div>

          <div v-if="form.avatarUrl" class="avatar-preview">
            <img :src="form.avatarUrl" alt="头像预览" @error="handleAvatarError" />
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
  phone: '',
  avatarUrl: '',
})

const fillForm = () => {
  form.name = authStore.user?.name || ''
  form.phone = authStore.user?.phone || ''
  form.avatarUrl = authStore.user?.avatar_url || ''
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

const handleAvatarError = () => {
  saveError.value = '头像地址不可访问，请检查 URL 是否正确'
}

const handleSave = async () => {
  saveError.value = null
  saveSuccess.value = null
  saving.value = true

  try {
    await authStore.updateProfile({
      name: form.name.trim() || undefined,
      phone: form.phone.trim(),
      avatar_url: form.avatarUrl.trim(),
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

.avatar-preview {
  margin-bottom: 18px;
  width: 72px;
  height: 72px;
  border-radius: 50%;
  overflow: hidden;
  border: 2px solid #e0d8cf;
}

.avatar-preview img {
  width: 100%;
  height: 100%;
  object-fit: cover;
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
