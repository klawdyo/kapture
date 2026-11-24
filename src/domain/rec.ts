// Estado da gravação (regras puras).
import type { RecState } from './types'

/** Estado inicial: parado. */
export const idle: RecState = { state: 'idle', secs: 0 }

/** Há gravação em curso (gravando ou pausada)? */
export function isActive(s: RecState): boolean {
  return s.state !== 'idle'
}

/** Gravação pausada? */
export function isPaused(s: RecState): boolean {
  return s.state === 'paused'
}
