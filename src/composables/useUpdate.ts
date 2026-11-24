// Lógica do aviso de atualização (dados e ações próprios).
import { computed, ref } from 'vue'
import { msg } from '../domain/err'
import type { Upd } from '../domain/types'
import { ports } from '../ports'
import type { LogLvl } from '../ports/types'

export function useUpdate() {
  const { backend: b, updates: u } = ports()
  const cur = ref('') // versão instalada
  const upd = ref<Upd | null>(null) // atualização encontrada
  const open = ref(false) // mensagem aberta
  const has = computed(() => upd.value !== null)

  /** Registra no log em disco; falhas do próprio log são descartadas. */
  const log = (lvl: LogLvl, text: string): void => {
    b.logUi(lvl, `[update] ${text}`).catch(() => {})
  }

  /**
   * Consulta os Releases uma única vez (chamada ao abrir a janela).
   * Sem atualização ou com falha de rede nada aparece: só o log registra.
   */
  async function init(): Promise<void> {
    try {
      cur.value = await b.version()
      upd.value = await u.check(cur.value)
      log('info', upd.value ? `atualização disponível v${upd.value.version}` : 'sem atualização')
    } catch (e) {
      upd.value = null
      log('warn', `falha ao verificar: ${msg(e)}`)
    }
  }

  /** Abre ou fecha a mensagem. */
  const toggle = (): void => { open.value = !open.value }
  /** "Depois": fecha a mensagem; a bolinha continua. */
  const later = (): void => { open.value = false }

  /** "Baixar": abre a página do release no navegador e fecha a mensagem. */
  async function download(): Promise<void> {
    if (!upd.value) return
    open.value = false
    try {
      await b.openUrl(upd.value.url)
    } catch (e) {
      log('warn', `falha ao abrir o release: ${msg(e)}`)
    }
  }

  return { cur, upd, open, has, init, toggle, later, download }
}
