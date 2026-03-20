<template>
  <div class="modal-overlay" @click="$emit('close')">
    <div class="modal-content" @click.stop>
      <div class="modal-header">
        <h2>创建新群</h2>
        <button class="close-btn" @click="$emit('close')">✕</button>
      </div>

      <form @submit.prevent="handleSubmit" class="modal-form">
        <div class="form-group">
          <label for="group-name">群名称</label>
          <input
            id="group-name"
            v-model="form.groupName"
            type="text"
            placeholder="请输入群名称"
            maxlength="50"
            required
            :disabled="loading"
          />
        </div>

        <div class="form-group">
          <label for="group-number">群号</label>
          <input
            id="group-number"
            v-model="form.groupNumber"
            type="text"
            placeholder="请输入群号"
            maxlength="20"
            required
            :disabled="loading"
          />
          <p class="help-text">群号必须唯一</p>
        </div>

        <div v-if="error" class="error-message">
          {{ error }}
        </div>

        <div class="form-actions">
          <button type="button" class="btn-cancel" @click="$emit('close')" :disabled="loading">
            取消
          </button>
          <button type="submit" class="btn-submit" :disabled="loading">
            {{ loading ? '创建中...' : '创建' }}
          </button>
        </div>
      </form>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref } from 'vue'

const emit = defineEmits<{
  create: [groupName: string, groupNumber: string]
  close: []
}>()

const form = ref({
  groupName: '',
  groupNumber: '',
})
const loading = ref(false)
const error = ref<string | null>(null)

const handleSubmit = async () => {
  if (!form.value.groupName.trim()) {
    error.value = '请输入群名称'
    return
  }

  if (!form.value.groupNumber.trim()) {
    error.value = '请输入群号'
    return
  }

  loading.value = true
  error.value = null

  try {
    emit('create', form.value.groupName, form.value.groupNumber)
  } catch (err: any) {
    error.value = err.message || '创建失败'
  } finally {
    loading.value = false
  }
}
</script>

<style scoped>
.modal-overlay {
  position: fixed;
  top: 0;
  left: 0;
  right: 0;
  bottom: 0;
  background-color: rgba(0, 0, 0, 0.3);
  display: flex;
  align-items: center;
  justify-content: center;
  z-index: 1000;
  padding: 20px;
}

.modal-content {
  background: white;
  border-radius: 8px;
  width: 100%;
  max-width: 500px;
  box-shadow: 0 4px 20px rgba(0, 0, 0, 0.15);
}

.modal-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 20px;
  border-bottom: 1px solid #d4cfc8;
}

.modal-header h2 {
  font-size: 18px;
  font-weight: 600;
  color: #4a4a4a;
  margin: 0;
}

.close-btn {
  background: none;
  border: none;
  font-size: 20px;
  cursor: pointer;
  color: #888888;
  transition: color 0.3s ease;
}

.close-btn:hover {
  color: #4a4a4a;
}

.modal-form {
  padding: 20px;
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

.help-text {
  font-size: 12px;
  color: #cccccc;
  margin: 4px 0 0 0;
}

.error-message {
  padding: 10px 12px;
  background-color: #f5e6e6;
  color: #a88b7f;
  border-radius: 6px;
  font-size: 13px;
  margin-bottom: 20px;
}

.form-actions {
  display: flex;
  gap: 10px;
  justify-content: flex-end;
}

.form-actions button {
  padding: 10px 20px;
  border: none;
  border-radius: 6px;
  font-size: 13px;
  font-weight: 500;
  cursor: pointer;
  transition: all 0.3s ease;
}

.btn-cancel {
  background-color: #d4cfc8;
  color: #4a4a4a;
}

.btn-cancel:hover:not(:disabled) {
  background-color: #e8e3dd;
}

.btn-submit {
  background-color: #8b9d83;
  color: white;
}

.btn-submit:hover:not(:disabled) {
  background-color: #9caa93;
}

.btn-cancel:disabled,
.btn-submit:disabled {
  opacity: 0.6;
  cursor: not-allowed;
}
</style>
