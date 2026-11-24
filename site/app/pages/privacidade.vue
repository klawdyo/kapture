<script setup lang="ts">
// Página de Política de Privacidade do kapture em português.
const config = useRuntimeConfig()

useSeoMeta({
  title: 'Política de Privacidade · kapture',
  description: 'O kapture não coleta dados pessoais nem telemetria. Tudo o que você grava permanece localmente no seu computador.'
})
</script>

<template>
  <article class="mx-auto max-w-3xl px-5 py-16 sm:px-8 lg:py-24">
    <NuxtLink
      to="/"
      class="focus-ring inline-flex items-center gap-1.5 text-sm text-[var(--ink-muted)] transition-colors hover:text-[var(--ink-primary)]"
    >
      <UIcon name="i-lucide-arrow-left" class="size-4" aria-hidden="true" />
      <span>Voltar ao início</span>
    </NuxtLink>

    <h1 class="font-display mt-6 text-3xl font-bold tracking-[-0.025em] text-[var(--ink-primary)] sm:text-4xl">
      Política de Privacidade
    </h1>
    <p class="mt-4 text-base leading-relaxed text-[var(--ink-secondary)]">
      O kapture é um gravador de tela local para Windows. Ele foi feito para funcionar sem coletar dados pessoais.
    </p>

    <div class="mt-10 space-y-8 border-t border-[var(--line-hairline)] pt-8 text-base leading-relaxed text-[var(--ink-secondary)]">
      <section>
        <h2 class="font-display text-lg font-bold text-[var(--ink-primary)]">
          Dados coletados
        </h2>
        <p class="mt-2">
          O kapture não coleta, não armazena remotamente e não envia nenhum dado pessoal, telemetria, estatística de uso, gravação de tela, áudio do microfone, imagem da webcam ou relatório de erro. Tudo o que você grava permanece exclusivamente na pasta escolhida no seu computador.
        </p>
      </section>

      <section>
        <h2 class="font-display text-lg font-bold text-[var(--ink-primary)]">
          Logs de diagnóstico
        </h2>
        <p class="mt-2">
          Para ajudar a investigar falhas técnicas, o kapture registra eventos operacionais localmente em
          <code class="rounded bg-[var(--line-hairline)] px-1.5 py-0.5 font-mono text-xs text-[var(--ink-primary)]">%LOCALAPPDATA%\kapture\logs\kapture.log</code>
          (no máximo 3 arquivos de 512 KB com rotação automática). Esses arquivos nunca são enviados pela internet e só saem do seu computador se você decidir compartilhá-los manualmente.
        </p>
      </section>

      <section>
        <h2 class="font-display text-lg font-bold text-[var(--ink-primary)]">
          Acesso à internet
        </h2>
        <p class="mt-2">
          Ao abrir o aplicativo, o kapture faz apenas uma consulta HTTPS comum ao GitHub Releases para verificar se existe uma versão mais recente disponível. Nenhum identificador pessoal é enviado além dos cabeçalhos padrão de uma requisição HTTPS. Se preferir bloquear até essa consulta, basta impedir a saída de rede do executável no Firewall do Windows.
        </p>
      </section>

      <section>
        <h2 class="font-display text-lg font-bold text-[var(--ink-primary)]">
          Contato
        </h2>
        <p class="mt-2">
          Dúvidas sobre esta política podem ser abertas nas
          <a class="link text-[var(--ink-primary)]" :href="`${config.public.repoUrl}/issues`" rel="noopener">issues do repositório no GitHub</a>.
        </p>
      </section>
    </div>
  </article>
</template>
