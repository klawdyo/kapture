<script setup lang="ts">
// Demonstração interativa construída diretamente com os mesmos componentes Vue do app (src/).
// Acompanha automaticamente o tema claro/escuro do site como se fosse o tema do sistema.
import { computed, onBeforeUnmount, onMounted, ref } from 'vue'
import DevSelect from '@app/components/DevSelect.vue'
import ModeSwitch from '@app/components/ModeSwitch.vue'
import RecBtn from '@app/components/RecBtn.vue'
import TitleBar from '@app/components/TitleBar.vue'
import VuMeter from '@app/components/VuMeter.vue'
import type { Dev, Mode } from '@app/domain/types'
import Box from '@app/ui/Box.vue'
import Btn from '@app/ui/Btn.vue'

// Dispositivos ilustrativos para simular o comportamento real da barra.
const demoCams: Dev[] = [
  { id: 'cam-1', name: 'Logitech StreamCam' },
  { id: 'cam-2', name: 'Webcam integrada' }
]
const demoMics: Dev[] = [
  { id: 'mic-1', name: 'HyperX QuadCast' },
  { id: 'mic-2', name: 'Microfone do notebook' }
]

const stageRef = ref<HTMLElement | null>(null)
const mode = ref<Mode>('screen')
const cam = ref('cam-1')
const mic = ref('mic-1')
const active = ref(false)
const saved = ref(false)
const secs = ref(0)
const lvl = ref(0.28)
const userControlled = ref(false)
const visible = ref(false)

let clockTimer: ReturnType<typeof setInterval> | null = null
let vuTimer: ReturnType<typeof setInterval> | null = null
let autoTimer: ReturnType<typeof setTimeout> | null = null
let savedTimer: ReturnType<typeof setTimeout> | null = null
let observer: IntersectionObserver | null = null

// Formata os segundos decorridos no formato MM:SS igual ao app.
const clock = computed(() => {
  const m = String(Math.floor(secs.value / 60)).padStart(2, '0')
  const s = String(secs.value % 60).padStart(2, '0')
  return `${m}:${s}`
})

// Limpa o agendador da demonstração automática.
function clearAuto() {
  if (autoTimer) {
    clearTimeout(autoTimer)
    autoTimer = null
  }
}

// Marca que a pessoa assumiu o controle manual e interrompe o ciclo automático.
function takeControl() {
  userControlled.value = true
  clearAuto()
}

// Inicia a simulação de gravação e encolhe a barra para a pílula compacta.
function startRec() {
  if (active.value) return
  saved.value = false
  secs.value = 0
  active.value = true
  if (clockTimer) clearInterval(clockTimer)
  clockTimer = setInterval(() => {
    secs.value += 1
  }, 1000)
}

// Encerra a gravação, expande a barra de volta e exibe o indicador "Salvo".
function stopRec() {
  if (!active.value) return
  active.value = false
  if (clockTimer) {
    clearInterval(clockTimer)
    clockTimer = null
  }
  saved.value = true
  if (savedTimer) clearTimeout(savedTimer)
  savedTimer = setTimeout(() => {
    saved.value = false
  }, 2400)
}

// Alterna gravação ao clicar no botão vermelho da barra ou da pílula.
function onToggleRec() {
  takeControl()
  if (active.value) stopRec()
  else startRec()
}

// Atualiza o modo de captura quando o usuário clica em Tela, Janela ou Área.
function onMode(next: Mode) {
  takeControl()
  mode.value = next
}

// Atualiza a câmera selecionada pelo usuário.
function onCam(next: string) {
  takeControl()
  cam.value = next
}

// Atualiza o microfone selecionado e zera o medidor quando desligado.
function onMic(next: string) {
  takeControl()
  mic.value = next
  if (!next) lvl.value = 0
}

// Executa um ciclo calmo da demonstração automática enquanto o bloco estiver visível.
function scheduleAuto() {
  clearAuto()
  if (userControlled.value || !visible.value) return
  if (window.matchMedia('(prefers-reduced-motion: reduce)').matches) return

  if (!active.value) {
    // Aguarda em repouso para a pessoa ver a barra completa e então inicia a gravação.
    autoTimer = setTimeout(() => {
      if (userControlled.value || !visible.value) return
      startRec()
      scheduleAuto()
    }, 2600)
  } else {
    // Mantém a pílula gravando por alguns segundos e depois retorna à barra completa.
    autoTimer = setTimeout(() => {
      if (userControlled.value || !visible.value) return
      stopRec()
      scheduleAuto()
    }, 4200)
  }
}

onMounted(() => {
  // Oscila suavemente o VU meter quando há microfone selecionado.
  vuTimer = setInterval(() => {
    if (!mic.value) {
      lvl.value = 0
      return
    }
    const base = active.value ? 0.52 : 0.28
    lvl.value = Math.min(0.92, Math.max(0.12, base + (Math.random() - 0.45) * 0.36))
  }, 220)

  // Observa quando o bloco entra na tela para iniciar a demonstração automática.
  if (typeof IntersectionObserver !== 'undefined' && stageRef.value) {
    observer = new IntersectionObserver(
      (entries) => {
        const entry = entries[0]
        visible.value = Boolean(entry?.isIntersecting)
        if (visible.value) scheduleAuto()
        else clearAuto()
      },
      { threshold: 0.35 }
    )
    observer.observe(stageRef.value)
  }
})

onBeforeUnmount(() => {
  clearAuto()
  if (clockTimer) clearInterval(clockTimer)
  if (vuTimer) clearInterval(vuTimer)
  if (savedTimer) clearTimeout(savedTimer)
  observer?.disconnect()
})
</script>

<template>
  <div ref="stageRef" class="kapture-stage">
    <!-- Área com altura estável para a barra encolher e expandir sem deslocar o conteúdo abaixo. -->
    <div class="kapture-canvas">
      <div class="kapture-scale">
        <Box class="bar" :class="{ pill: active }">
          <template v-if="active">
            <Box class="dot" />
            <Box tag="span" class="clock">{{ clock }}</Box>
            <RecBtn :rec="true" @click="onToggleRec" />
          </template>
          <template v-else>
            <TitleBar @min="takeControl" @close="takeControl">
              <Box v-if="saved" tag="span" class="ok">Salvo</Box>
            </TitleBar>
            <Box class="row">
              <BrandLogo :size="64" alt="kapture" class="logo [&_img]:size-full" />
              <Box class="sep" />
              <ModeSwitch :model-value="mode" @update:model-value="onMode" />
              <Box class="sep" />
              <DevSelect :model-value="cam" :devs="demoCams" icon="cam" label="Câmera" none="Sem câmera" @update:model-value="onCam" />
              <DevSelect :model-value="mic" :devs="demoMics" icon="mic" label="Microfone" none="Sem microfone" @update:model-value="onMic">
                <VuMeter :value="lvl" />
              </DevSelect>
              <Box class="sep" />
              <RecBtn :rec="false" @click="onToggleRec" />
              <Btn class="gear" round icon="gear" title="Configurações" @click="takeControl" />
            </Box>
          </template>
        </Box>
      </div>
    </div>

    <p class="mt-3 text-center text-xs text-[var(--ink-muted)]">
      Clique em Gravar para testar a troca entre a barra e a pílula.
    </p>
  </div>
</template>

<style scoped>
.kapture-canvas {
  display: grid;
  place-items: center;
  min-height: 148px;
  padding: 0.75rem 0;
}

.kapture-scale {
  display: grid;
  place-items: center;
  width: 760px;
  height: 118px;
  font-family: 'Segoe UI', system-ui, sans-serif;
  font-size: 13px;
  color: var(--fg);
  user-select: none;
}

/* Barra real do app com transição suave de dimensão para o modo pílula compacta. */
.bar {
  position: relative;
  box-sizing: border-box;
  width: 760px;
  height: 118px;
  display: flex;
  flex-direction: column;
  border-radius: 10px;
  background: var(--bg);
  border: 1px solid var(--bd);
  overflow: hidden;
  backdrop-filter: blur(18px);
  box-shadow: var(--bar-shadow);
  transition:
    width 360ms cubic-bezier(0.22, 1, 0.36, 1),
    height 360ms cubic-bezier(0.22, 1, 0.36, 1),
    border-radius 360ms cubic-bezier(0.22, 1, 0.36, 1),
    padding 360ms cubic-bezier(0.22, 1, 0.36, 1),
    background-color 200ms ease,
    border-color 200ms ease,
    opacity 180ms ease;
}

.row {
  display: flex;
  align-items: center;
  gap: 9px;
  padding: 4px 14px 10px 12px;
  flex: 1;
  min-height: 0;
}

.gear { color: var(--fg); }
.logo { flex: none; width: 46px; height: 46px; }
.sep { flex: none; width: 1px; height: 56px; background: var(--sep); }

/* Pílula compacta durante a gravação, acompanhando o tema claro ou escuro. */
.pill {
  width: 150px;
  height: 50px;
  flex-direction: row;
  align-items: center;
  gap: 8px;
  padding: 0 7px 0 16px;
  border-radius: 25px;
  background: var(--pill-bg);
  opacity: 0.92;
}

.pill:hover,
.pill:focus-within {
  opacity: 1;
}

.dot {
  width: 10px;
  height: 10px;
  border-radius: 50%;
  background: var(--red);
  animation: kapture-blink 1.2s infinite;
}

.clock {
  flex: 1;
  min-width: 0;
  font-variant-numeric: tabular-nums;
  font-weight: 600;
  font-size: 16px;
  color: var(--fg);
}

.ok {
  color: var(--ok);
  font-size: 12px;
  font-weight: 600;
}

@keyframes kapture-blink {
  50% { opacity: 0.3; }
}

@media (prefers-reduced-motion: reduce) {
  .bar { transition: none; }
  .dot { animation: none; }
}

/* Redução proporcional em telas menores que 800px sem overflow horizontal. */
@media (max-width: 820px) {
  .kapture-canvas {
    min-height: 112px;
    overflow: hidden;
  }
  .kapture-scale {
    transform: scale(0.72);
    transform-origin: center;
  }
}

@media (max-width: 580px) {
  .kapture-canvas {
    min-height: 92px;
  }
  .kapture-scale {
    transform: scale(0.46);
  }
}
</style>
