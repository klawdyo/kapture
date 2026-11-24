// Entrada da janela: só monta o app (sem lógica).
import { boot } from './boot'
import './styles.css'
import WebcamApp from './windows/WebcamApp.vue'

boot(WebcamApp, 'webcam')
