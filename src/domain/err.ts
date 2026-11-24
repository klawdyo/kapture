// Normalização de erros vindos do backend (strings em português) ou do JS.

/** Extrai mensagem legível de qualquer erro. */
export function msg(e: unknown): string {
  if (typeof e === 'string') return e
  if (e instanceof Error) return e.message
  return 'Erro desconhecido'
}
