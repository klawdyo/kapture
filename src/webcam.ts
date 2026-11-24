// Entrada da janela: só monta o app (sem lógica).
import { createApp } from 'vue'
import './styles.css'
import WebcamApp from './windows/WebcamApp.vue'

createApp(WebcamApp).mount('#app')
