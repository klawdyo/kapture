// Texto das mensagens de erro da interface enviadas ao log em disco.
import { detail } from './err'

/** Parte do `ErrorEvent` que usamos (facilita testar sem o DOM). */
export interface ErrInfo {
  message: string
  filename?: string
  lineno?: number
  colno?: number
  error?: unknown
}

/** Erro não tratado (`window.onerror`): mensagem, local e pilha. */
export function fmtError(e: ErrInfo): string {
  const at = e.filename ? ` em ${e.filename}:${e.lineno ?? 0}:${e.colno ?? 0}` : ''
  const stack = e.error instanceof Error && e.error.stack ? `\n${e.error.stack}` : ''
  return `erro não tratado: ${e.message}${at}${stack}`
}

/** Promessa rejeitada sem `catch`. */
export const fmtReject = (reason: unknown): string => `promessa rejeitada: ${detail(reason)}`

/** Erro lançado dentro de um componente Vue. */
export const fmtVue = (e: unknown, info: string): string => `erro no componente (${info}): ${detail(e)}`
