import { createApp } from 'vue'
import { createPinia } from 'pinia'
import router from './router'
import App from './App.vue'
import { applyGlobalStyles } from './theme/styles'
import './globals.css'

// 应用莫兰迪色系全局样式
applyGlobalStyles()

const app = createApp(App)

// 注册插件
app.use(createPinia())
app.use(router)

// 挂载应用
app.mount('#app')
