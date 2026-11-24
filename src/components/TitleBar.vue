<!-- Barra de título do painel: logo, nome, status, minimizar e fechar. Arrasta a janela. -->
<script setup lang="ts">
import logo from '../assets/logo.png'
import Btn from '../ui/Btn.vue'

defineEmits<{ min: []; close: [] }>()
</script>

<template>
  <div class="tb" data-tauri-drag-region>
    <img :src="logo" width="16" height="16" alt="" draggable="false" data-tauri-drag-region />
    <span class="nm" data-tauri-drag-region><b>Cap</b> • Gravador de Tela</span>
    <span class="st" data-tauri-drag-region><slot /></span>
    <Btn icon="min" :size="16" title="Minimizar" @click="$emit('min')" />
    <Btn icon="close" :size="16" title="Fechar" @click="$emit('close')" />
  </div>
</template>

<style scoped>
.tb { display: flex; align-items: center; gap: 8px; height: 32px; padding: 0 6px 0 12px; }
img { border-radius: 3px; }
.nm { font-size: 12px; color: var(--mut); white-space: nowrap; }
.nm b { color: var(--fg); font-weight: 600; }
.st { flex: 1; min-width: 0; display: flex; justify-content: flex-end; }
.tb :deep(.btn) { width: 40px; height: 28px; padding: 0; border-radius: 6px; color: var(--fg); }
.tb :deep(.btn:last-child:hover) { background: #c42b1c; }
</style>
