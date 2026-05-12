import { createApp } from 'vue'
import App from './App.vue'
import router from './router'
import i18n from './i18n'
import './style.css'
import 'md-editor-v3/lib/preview.css'

const app = createApp(App)
app.use(router)
app.use(i18n)
app.mount('#app')
