// Entrada da janela: só monta o app (sem lógica).
import { createApp } from 'vue'
import './styles.css'
import RegionSelector from './windows/RegionSelector.vue'

createApp(RegionSelector).mount('#app')
