// Ponto único de acesso às portas; testes trocam por FAKEs com setPorts.
import { realBackend, realMedia, realUpdates } from './real'
import type { Ports } from './types'

let cur: Ports = { backend: realBackend, media: realMedia, updates: realUpdates }

/** Portas atuais (REAIS por padrão). */
export const ports = (): Ports => cur

/** Substitui as portas (usado em testes). */
export const setPorts = (p: Ports): void => {
  cur = p
}
