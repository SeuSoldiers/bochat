<template>
  <router-view />
</template>

<script setup lang="ts">
</script>

<style>
html,
body {
  margin: 0;
  padding: 0;
  width: 100%;
  height: 100%;
}

body {
  background-color: var(--color-bg, #f5f3f1);
}

#app {
  width: 100%;
  min-height: 100vh;
}
</style>
