// Regras puras da verificação de atualização (sem rede nem Tauri).
import type { Upd } from './types'

/** Fase do aviso de atualização na interface. */
export type Phase = 'idle' | 'checking' | 'downloading' | 'ready' | 'available'

/** Texto do status por fase; `pct` (0..100) aparece só durante o download e quando maior que zero. */
export function updText(phase: Phase, pct = 0): string {
  switch (phase) {
    case 'checking': return 'Verificando atualizações…'
    case 'downloading': return pct > 0 ? `Atualizando… ${Math.round(pct)}%` : 'Atualizando…'
    case 'ready': return 'Reinicie para atualizar'
    case 'available': return 'Atualização disponível'
    default: return ''
  }
}

/** Prefixo das únicas páginas de release aceitas (igual ao backend). */
export const REL_URL = 'https://github.com/klawdyo/kapture/releases/'

/** Versão decomposta: maior, menor, correção. */
export type Ver = [number, number, number]

/**
 * Lê uma tag como versão, ignorando o prefixo 'v'. Aceita só X.Y.Z
 * (metadados de build `+x` são ignorados); prerelease (`-beta`) e
 * qualquer outro formato viram null.
 */
export function parseVer(tag: string): Ver | null {
  const m = /^[vV]?(\d+)\.(\d+)\.(\d+)(?:\+[0-9A-Za-z.-]+)?$/.exec(tag.trim())
  return m ? [Number(m[1]), Number(m[2]), Number(m[3])] : null
}

/** Compara versões: negativo se a < b, 0 se iguais, positivo se a > b. */
export function cmpVer(a: Ver, b: Ver): number {
  return a[0] - b[0] || a[1] - b[1] || a[2] - b[2]
}

/** A URL é uma página de release do kapture (sem espaços nem '..')? */
export function okUrl(url: string): boolean {
  return url.startsWith(REL_URL) && !url.includes('..') && !/[\s"\\]/.test(url)
}

/** Texto da versão sem o prefixo 'v' (ex.: 'v1.2.3' vira '1.2.3'). */
export const bare = (tag: string): string => tag.trim().replace(/^[vV]/, '')

/**
 * Interpreta a resposta de `releases/latest` do GitHub. Devolve a
 * atualização candidata ou null se faltar campo, for rascunho,
 * prerelease, tag fora do formato ou URL fora do repositório.
 */
export function parseRel(data: unknown): Upd | null {
  if (typeof data !== 'object' || data === null) return null
  const d = data as Record<string, unknown>
  if (typeof d.tag_name !== 'string' || typeof d.html_url !== 'string') return null
  if (d.draft === true || d.prerelease === true) return null
  if (!parseVer(d.tag_name) || !okUrl(d.html_url)) return null
  return { version: bare(d.tag_name), url: d.html_url }
}

/** Entre os candidatos, o mais novo que for maior que `cur`; senão null. */
export function pickNew(cur: string, list: Upd[]): Upd | null {
  const base = parseVer(cur)
  if (!base) return null
  let best: Upd | null = null
  let top = base
  for (const u of list) {
    const v = parseVer(u.version)
    if (v && cmpVer(v, top) > 0) {
      best = u
      top = v
    }
  }
  return best
}
