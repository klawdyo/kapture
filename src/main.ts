// Entrada da janela: só monta o app (sem lógica).
import { createApp } from 'vue'
import './styles.css'
import ToolbarApp from './windows/ToolbarApp.vue'

createApp(ToolbarApp).mount('#app')
