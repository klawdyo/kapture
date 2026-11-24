// Montagem da configuração de gravação e opções de alvo.
import type { Mode, Opt, RecCfg, Region, Targets } from './types'

/** Opções de quadros por segundo. */
export const FPS: Opt[] = ['15', '24', '30', '60'].map((v) => ({ value: v, label: `${v} fps` }))

/** Entradas para montar a configuração. */
export interface CfgIn {
  mode: Mode
  tid: string
  mic: string
  sys: boolean
  fps: number
  region?: Region
}

/** Monta o RecCfg omitindo campos vazios. */
export function buildCfg(i: CfgIn): RecCfg {
  const c: RecCfg = { mode: i.mode, sys: i.sys, fps: i.fps }
  if (i.mode !== 'area' && i.tid !== '') c.targetId = Number(i.tid)
  if (i.mic !== '') c.micId = i.mic
  if (i.region) c.region = i.region
  return c
}

/** Opções do seletor de alvo conforme o modo (área não tem alvo). */
export function targetOpts(mode: Mode, t: Targets): Opt[] {
  if (mode === 'screen') {
    return t.monitors.map((m) => ({ value: String(m.id), label: `${m.name} (${m.w}×${m.h})` }))
  }
  if (mode === 'window') return t.wins.map((w) => ({ value: String(w.id), label: w.title }))
  return []
}
