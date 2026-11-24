<!-- Painel da barra de ferramentas. Sem lógica: tudo vem de useToolbar. -->
<script setup lang="ts">
import { onBeforeUnmount, onMounted, reactive } from 'vue'
import DevSelect from '../components/DevSelect.vue'
import ModeSwitch from '../components/ModeSwitch.vue'
import RecBtn from '../components/RecBtn.vue'
import TitleBar from '../components/TitleBar.vue'
import VuMeter from '../components/VuMeter.vue'
import { useToolbar } from '../composables/useToolbar'
import logo from '../assets/logo.png'
import { FPS } from '../domain/cfg'
import Box from '../ui/Box.vue'
import Btn from '../ui/Btn.vue'
import Select from '../ui/Select.vue'

const t = reactive(useToolbar())
onMounted(t.init)
onBeforeUnmount(t.dispose)
</script>

<template>
  <Box class="bar" :class="{ pill: t.active }" data-tauri-drag-region>
    <!-- Modo compacto durante a gravação -->
    <template v-if="t.active">
      <Box class="dot" :class="{ off: t.paused }" data-tauri-drag-region />
      <Box tag="span" class="clock" data-tauri-drag-region>{{ t.clock }}</Box>
      <Box v-if="t.err" tag="span" class="err" :title="t.err">{{ t.err }}</Box>
      <Box class="fill" data-tauri-drag-region />
      <Btn class="pz" round :icon="t.paused ? 'play' : 'pause'" :title="t.paused ? 'Retomar' : 'Pausar'" @click="t.pause()" />
      <RecBtn :rec="true" @click="t.toggle()" />
    </template>
    <template v-else>
      <TitleBar @min="t.minimize()" @close="t.close()">
        <Box v-if="t.err" tag="span" class="err" :title="t.err" data-tauri-drag-region>{{ t.err }}</Box>
        <Box v-else-if="t.saved" tag="span" class="ok" :title="t.saved" data-tauri-drag-region>Salvo</Box>
      </TitleBar>
      <Box class="row" data-tauri-drag-region>
        <img :src="logo" class="logo" width="46" height="46" alt="Cap" draggable="false" data-tauri-drag-region />
        <Box class="sep" data-tauri-drag-region />
        <ModeSwitch :model-value="t.mode" @update:model-value="t.setMode" />
        <Box class="sep" data-tauri-drag-region />
        <DevSelect :model-value="t.cam" :devs="t.devs.cams" icon="cam" label="Câmera" none="Sem câmera" @update:model-value="t.setCam" />
        <DevSelect :model-value="t.mic" :devs="t.devs.mics" icon="mic" label="Microfone" none="Sem microfone" @update:model-value="t.setMic">
          <VuMeter :value="t.lvl" />
        </DevSelect>
        <Box class="sep" data-tauri-drag-region />
        <RecBtn :rec="false" @click="t.toggle()" />
        <Btn class="gear" round icon="gear" :on="t.cfg" title="Configurações" @click="t.toggleCfg()" />
      </Box>
      <!-- Linha extra: alvo (várias telas/janelas) e configurações -->
      <Box v-if="t.extra" class="extra" data-tauri-drag-region>
        <Select v-if="t.opts.length" v-model="t.tid" class="tgt" :options="t.opts" title="Alvo" />
        <template v-if="t.cfg">
          <Btn icon="sys" :size="16" :on="t.sys" title="Áudio do sistema" @click="t.toggleSys()">Áudio do sistema</Btn>
          <Select v-model="t.fps" class="fps" :options="FPS" title="Quadros por segundo" />
          <Btn title="Abrir pasta de logs" @click="t.openLogs()">Abrir logs</Btn>
        </template>
      </Box>
    </template>
  </Box>
</template>

<style scoped>
/* Painel ocupa a janela inteira: o mica e a sombra vêm do sistema; o fundo
   semi-opaco abaixo é o fallback quando o efeito não se aplica. */
.bar {
  box-sizing: border-box; width: 100%; height: 100%; display: flex; flex-direction: column;
  border-radius: 10px; background: var(--bg); border: 1px solid var(--bd); overflow: hidden;
  backdrop-filter: blur(18px);
  box-shadow: inset 0 1px 0 rgba(255, 255, 255, 0.06);
}
.row { display: flex; align-items: center; gap: 9px; padding: 4px 14px 10px 12px; flex: 1; min-height: 0; }
.gear { color: var(--fg); }
.logo { flex: none; width: 46px; height: 46px; }
.sep { flex: none; width: 1px; height: 56px; background: rgba(255, 255, 255, 0.14); }
.extra { display: flex; align-items: center; gap: 10px; height: 48px; padding: 0 16px; border-top: 1px solid rgba(255, 255, 255, 0.1); flex: none; }
.tgt { flex: 1 1 0; min-width: 0; max-width: 260px; }
.fps { width: 96px; }
.extra :deep(.btn) { border: 1px solid rgba(255, 255, 255, 0.1); background: var(--fld); color: var(--fg); height: 30px; }
.extra :deep(.btn.on) { background: rgba(34, 197, 94, 0.22); border-color: rgba(34, 197, 94, 0.6); }
.pill { flex-direction: row; align-items: center; gap: 10px; padding: 0 10px 0 16px; border-radius: 28px; }
.dot { width: 10px; height: 10px; border-radius: 50%; background: var(--red); animation: blink 1.2s infinite; }
.dot.off { animation: none; background: var(--mut); }
.clock { min-width: 48px; font-variant-numeric: tabular-nums; font-weight: 600; font-size: 15px; }
.fill { flex: 1; align-self: stretch; }
.pz { background: rgba(255, 255, 255, 0.16); color: var(--fg); }
.pz:hover { background: rgba(255, 255, 255, 0.26); }
.err { color: #fca5a5; max-width: 260px; overflow: hidden; text-overflow: ellipsis; white-space: nowrap; font-size: 12px; }
.ok { color: #86efac; font-size: 12px; }
@keyframes blink { 50% { opacity: 0.3; } }
</style>
