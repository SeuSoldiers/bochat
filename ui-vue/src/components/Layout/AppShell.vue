<template>
  <div class="app-shell">
    <GlobalSidebar />
    <main class="app-main">
      <router-view />
    </main>
  </div>
</template>

<script setup lang="ts">
import GlobalSidebar from './GlobalSidebar.vue'
</script>

<style scoped>
.app-shell {
  display: flex;
  width: 100vw;
  height: 100vh;
  overflow: hidden;
}

.app-main {
  flex: 1;
  min-width: 0;
  overflow-y: auto;
  background: #f2f2f2;
  padding: 16px;
}

@media (max-width: 768px) {
  .app-shell {
    flex-direction: column;
  }

  .app-main {
    padding: 10px;
  }
}
</style>
