// Lógica do aviso de atualização: combina o ciclo automático do backend
// (verifica, baixa sozinho, pede reinício) com o aviso simples por consulta
// ao GitHub, usado quando o app não foi instalado pelo Setup (fallback).
import { computed, ref } from 'vue'
import { msg } from '../domain/err'
import type { Upd, UpdSt } from '../domain/types'
import { updText, type Phase } from '../domain/upd'
import { ports } from '../ports'
import type { LogLvl, Off } from '../ports/types'

export function useUpdate() {
  const { backend: b, updates: u } = ports()
  const cur = ref('') // versão instalada
  const upd = ref<Upd | null>(null) // atualização achada pelo fallback
  const open = ref(false) // mensagem do fallback aberta
  const phase = ref<Phase>('idle')
  const pct = ref(0) // progresso do download, 0..100
  const has = computed(() => upd.value !== null)
  const text = computed(() => updText(phase.value, pct.value))
  let off: Off | null = null

  /** Registra no log em disco; falhas do próprio log são descartadas. */
  const log = (lvl: LogLvl, t: string): void => {
    b.logUi(lvl, `[update] ${t}`).catch(() => {})
  }

  /** Fallback: consulta o último release no GitHub (uma requisição). */
  async function probe(): Promise<void> {
    phase.value = 'checking'
    try {
      upd.value = await u.check(cur.value)
      phase.value = upd.value ? 'available' : 'idle'
      log('info', upd.value ? `atualização disponível v${upd.value.version}` : 'sem atualização')
    } catch (e) {
      upd.value = null
      phase.value = 'idle'
      log('warn', `falha ao verificar: ${msg(e)}`)
    }
  }

  /** Reage a um estado do ciclo do backend. */
  function onState(s: UpdSt): void {
    if (s.kind === 'checking') phase.value = 'checking'
    else if (s.kind === 'downloading') {
      phase.value = 'downloading'
      pct.value = s.pct
    } else if (s.kind === 'ready') phase.value = 'ready'
    else if (s.kind === 'fallback') void probe()
    else phase.value = 'idle' // none, err (o backend já registrou) e idle: sem alarde
  }

  /**
   * Chamada ao abrir a janela: assina o ciclo do backend e o inicia (ele só
   * roda uma vez). Se o backend não responder, usa o aviso simples.
   */
  async function init(): Promise<void> {
    try {
      cur.value = await b.version()
    } catch (e) {
      log('warn', `falha ao verificar: ${msg(e)}`)
      return
    }
    try {
      off = await b.onUpdate(onState)
      await b.updateStart()
    } catch (e) {
      log('warn', `ciclo automático indisponível: ${msg(e)}`)
      await probe()
    }
  }

  /** Cancela a assinatura do evento. */
  const dispose = (): void => {
    off?.()
    off = null
  }

  /** Abre ou fecha a mensagem do fallback. */
  const toggle = (): void => { open.value = !open.value }
  /** "Depois": fecha a mensagem; a bolinha continua. */
  const later = (): void => { open.value = false }

  /** "Baixar" (fallback): abre a página do release no navegador e fecha a mensagem. */
  async function download(): Promise<void> {
    if (!upd.value) return
    open.value = false
    try {
      await b.openUrl(upd.value.url)
    } catch (e) {
      log('warn', `falha ao abrir o release: ${msg(e)}`)
    }
  }

  /** "Reinicie para atualizar": o backend aplica a versão baixada e reabre o app. */
  async function apply(): Promise<void> {
    try {
      await b.updateApply()
    } catch (e) {
      log('warn', `falha ao reiniciar: ${msg(e)}`)
    }
  }

  return { cur, upd, open, has, phase, pct, text, init, dispose, toggle, later, download, apply }
}
