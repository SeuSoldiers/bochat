<template>
  <div class="modal-overlay" @click="$emit('close')">
    <div class="modal-content" @click.stop>
      <div class="modal-header">
        <h2>创建新 Bot</h2>
        <button class="close-btn" @click="$emit('close')">✕</button>
      </div>

      <form @submit.prevent="handleSubmit" class="modal-form">
        <div class="form-group">
          <label for="bot-name">Bot 名称</label>
          <input
            id="bot-name"
            v-model="form.name"
            type="text"
            placeholder="请输入 Bot 名称"
            maxlength="50"
            required
            :disabled="loading"
          />
          <span class="char-count">{{ form.name.length }}/50</span>
        </div>

        <div class="form-group">
          <label for="bot-description">描述 (可选)</label>
          <textarea
            id="bot-description"
            v-model="form.description"
            placeholder="描述此 Bot 的功能和用途"
            maxlength="200"
            rows="3"
            :disabled="loading"
          />
          <span class="char-count">{{ form.description.length }}/200</span>
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
  create: [name: string, description: string]
  close: []
}>()

const form = ref({
  name: '',
  description: '',
})
const loading = ref(false)
const error = ref<string | null>(null)

const handleSubmit = async () => {
  if (!form.value.name.trim()) {
    error.value = '请输入 Bot 名称'
    return
  }

  loading.value = true
  error.value = null

  try {
    emit('create', form.value.name, form.value.description)
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
  position: relative;
}

.form-group label {
  display: block;
  margin-bottom: 6px;
  font-size: 13px;
  color: #4a4a4a;
  font-weight: 500;
}

.form-group input,
.form-group textarea {
  width: 100%;
  padding: 10px 12px;
  border: 1px solid #d4cfc8;
  border-radius: 6px;
  font-size: 14px;
  color: #4a4a4a;
  font-family: inherit;
  transition: all 0.3s ease;
  resize: vertical;
}

.form-group input:focus,
.form-group textarea:focus {
  outline: none;
  border-color: #8b9d83;
  box-shadow: 0 0 0 3px rgba(139, 157, 131, 0.1);
}

.form-group input:disabled,
.form-group textarea:disabled {
  background-color: #fafaf8;
  color: #cccccc;
  cursor: not-allowed;
}

.char-count {
  position: absolute;
  right: 12px;
  bottom: 8px;
  font-size: 12px;
  color: #cccccc;
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
