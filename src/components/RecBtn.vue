<!-- Botão circular vermelho: Gravar / Parar. -->
<script setup lang="ts">
import Btn from '../ui/Btn.vue'

defineProps<{ rec: boolean }>()
</script>

<template>
  <Btn kind="rec" round :icon="rec ? 'stop' : 'rec'" :title="rec ? 'Parar' : 'Gravar'" />
</template>
