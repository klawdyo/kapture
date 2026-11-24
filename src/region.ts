// Entrada da janela: só monta o app (sem lógica).
import { boot } from './boot'
import './styles.css'
import RegionSelector from './windows/RegionSelector.vue'

boot(RegionSelector, 'region')
