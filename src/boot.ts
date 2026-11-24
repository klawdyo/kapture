// Inicialização comum das janelas: monta o app e envia ao log em disco
// os erros que o WebView engoliria (onerror, promessas, componentes).
import { createApp, type Component } from 'vue'
import { fmtError, fmtReject, fmtVue } from './domain/log'
import { ports } from './ports'

/** Máximo de mensagens por janela (evita inundar o arquivo num laço de erros). */
export const MAX_LOGS = 100

/**
 * Assina os erros globais de `target` e os envia ao backend.
 * Devolve a função de envio (reaproveitada pelo `errorHandler` do Vue).
 */
export function installErrorLog(tag: string, target: EventTarget): (msg: string) => void {
  let sent = 0
  const send = (msg: string): void => {
    if (sent > MAX_LOGS) return
    sent++
    const text = sent > MAX_LOGS ? 'limite de mensagens de erro atingido; o restante é ignorado' : msg
    ports().backend.logUi('error', `[${tag}] ${text}`).catch(() => {})
  }
  target.addEventListener('error', (e) => send(fmtError(e as ErrorEvent)))
  target.addEventListener('unhandledrejection', (e) => send(fmtReject((e as PromiseRejectionEvent).reason)))
  return send
}

/** Monta o componente raiz em `#app` já com o log de erros ligado. */
export function boot(root: Component, tag: string): void {
  const app = createApp(root)
  const send = installErrorLog(tag, window)
  app.config.errorHandler = (e, _vm, info) => send(fmtVue(e, info))
  app.mount('#app')
}
