// Normalização de erros vindos do backend (strings em português) ou do JS.

/** Extrai mensagem legível de qualquer erro. */
export function msg(e: unknown): string {
  if (typeof e === 'string') return e
  if (e instanceof Error) return e.message
  return 'Erro desconhecido'
}

/** Texto completo para o log: pilha (quando existe) ou "Nome: mensagem". */
export function detail(e: unknown): string {
  if (e instanceof Error) return e.stack ?? `${e.name}: ${e.message}`
  try {
    return typeof e === 'string' ? e : (JSON.stringify(e) ?? String(e))
  } catch {
    return String(e) // circular, BigInt...
  }
}
