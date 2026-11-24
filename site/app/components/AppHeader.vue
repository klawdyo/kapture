<script setup lang="ts">
// Cabeçalho fixo enxuto com navegação principal, troca de tema e ação de download.
const config = useRuntimeConfig()
const menuOpen = ref(false)

const navItems = [
  { label: 'Recursos', href: '/#recursos' },
  { label: 'Instalação', href: '/#instalar' }
]
</script>

<template>
  <header class="sticky top-0 z-40 border-b border-[var(--line-hairline)] bg-[var(--surface-page)]/92 backdrop-blur-md">
    <div class="mx-auto flex h-16 max-w-6xl items-center justify-between gap-6 px-5 sm:px-8">
      <!-- Marca: logo oficial e wordmark minúsculo em vermelho. -->
      <NuxtLink to="/" class="focus-ring inline-flex items-center gap-2.5">
        <BrandLogo :size="32" />
        <span class="wordmark">kapture</span>
      </NuxtLink>

      <!-- Navegação principal em telas médias e grandes. -->
      <nav aria-label="Principal" class="hidden items-center gap-8 md:flex">
        <a
          v-for="item in navItems"
          :key="item.href"
          :href="item.href"
          class="focus-ring text-sm font-medium text-[var(--ink-secondary)] transition-colors hover:text-[var(--ink-primary)]"
        >
          {{ item.label }}
        </a>
      </nav>

      <!-- Ações à direita: alternador de tema e botão de download. -->
      <div class="flex items-center gap-2.5">
        <UColorModeButton aria-label="Alternar tema claro e escuro" />

        <a
          :href="config.public.downloadUrl"
          rel="noopener"
          class="focus-ring hidden items-center gap-2 rounded-lg bg-[#14181f] px-4 py-2 text-sm font-semibold text-[#f6f7f9] transition-colors hover:bg-[#262d38] dark:bg-[#f2f4f8] dark:text-[#15181e] dark:hover:bg-[#dce1ea] sm:inline-flex"
        >
          <UIcon name="i-lucide-download" class="size-4" aria-hidden="true" />
          <span>Baixar instalador</span>
        </a>

        <!-- Botão de menu móvel. -->
        <button
          type="button"
          class="focus-ring inline-flex size-10 items-center justify-center rounded-lg border border-[var(--line-hairline)] text-[var(--ink-primary)] md:hidden"
          :aria-expanded="menuOpen"
          aria-label="Abrir menu de navegação"
          @click="menuOpen = !menuOpen"
        >
          <UIcon :name="menuOpen ? 'i-lucide-x' : 'i-lucide-menu'" class="size-5" aria-hidden="true" />
        </button>
      </div>
    </div>

    <!-- Menu recolhível em telas pequenas. -->
    <div v-if="menuOpen" class="border-t border-[var(--line-hairline)] bg-[var(--surface-page)] px-5 py-4 md:hidden">
      <nav aria-label="Principal móvel" class="flex flex-col gap-3">
        <a
          v-for="item in navItems"
          :key="item.href"
          :href="item.href"
          class="focus-ring py-1.5 text-base font-medium text-[var(--ink-primary)]"
          @click="menuOpen = false"
        >
          {{ item.label }}
        </a>
        <a
          :href="config.public.downloadUrl"
          rel="noopener"
          class="focus-ring mt-2 inline-flex items-center justify-center gap-2 rounded-lg bg-[#14181f] px-4 py-2.5 text-sm font-semibold text-[#f6f7f9] dark:bg-[#f2f4f8] dark:text-[#15181e]"
          @click="menuOpen = false"
        >
          <UIcon name="i-lucide-download" class="size-4" aria-hidden="true" />
          <span>Baixar instalador</span>
        </a>
      </nav>
    </div>
  </header>
</template>
