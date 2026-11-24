// Entrada da janela: só monta o app (sem lógica).
import { boot } from './boot'
import './styles.css'
import ToolbarApp from './windows/ToolbarApp.vue'

boot(ToolbarApp, 'toolbar')
